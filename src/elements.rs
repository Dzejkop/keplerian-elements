use crate::math::newton_approx;
use crate::{vec3, Mat3, Num, StateVectors, Vec3, G, PI, TWO_PI};

#[derive(Debug, Clone, Copy)]
pub struct KeplerianElements {
    pub eccentricity: Num,
    pub semi_major_axis: Num,
    pub inclination: Num,
    pub right_ascension_of_the_ascending_node: Num,
    pub argument_of_periapsis: Num,
    pub mean_anomaly_at_epoch: Num,
    pub epoch: Num,
}
impl KeplerianElements {
    pub fn angle_abs_diff(&self, other: &Self) -> Num {
        let mut diff = 0.0;

        diff += (self.eccentricity - other.eccentricity).abs();
        diff += (self.inclination - other.inclination).abs();
        diff += (self.right_ascension_of_the_ascending_node
            - other.right_ascension_of_the_ascending_node)
            .abs();
        diff += (self.argument_of_periapsis - other.argument_of_periapsis).abs();
        diff += (self.mean_anomaly_at_epoch - other.mean_anomaly_at_epoch).abs();

        diff
    }

    pub fn from_state_vectors(state_vectors: &StateVectors, mass: Num, time: Num) -> Self {
        state_vectors.to_elements(mass, time)
    }

    pub fn ascending_node(&self, mass: Num) -> Vec3 {
        self.position_at_true_anomaly(mass, -self.argument_of_periapsis)
    }

    pub fn descending_node(&self, mass: Num) -> Vec3 {
        self.position_at_true_anomaly(mass, PI - self.argument_of_periapsis)
    }

    pub fn periapsis(&self, mass: Num) -> Vec3 {
        self.position_at_true_anomaly(mass, 0.0)
    }

    pub fn apoapsis(&self, mass: Num) -> Vec3 {
        self.position_at_true_anomaly(mass, PI)
    }

    pub fn normal(&self) -> Vec3 {
        self.perifocal_to_equatorial(Vec3::Z)
    }

    /// https://en.wikipedia.org/wiki/Standard_gravitational_parameter
    pub fn standard_gravitational_parameter(mass: Num) -> Num {
        G * mass
    }

    /// https://en.wikipedia.org/wiki/Orbital_period
    pub fn period(&self, mass: Num) -> Num {
        Self::period_static(self.semi_major_axis, mass)
    }

    pub fn period_static(a: Num, mass: Num) -> Num {
        TWO_PI * (a.powi(3) / Self::standard_gravitational_parameter(mass)).sqrt()
    }

    /// https://en.wikipedia.org/wiki/Mean_anomaly
    pub fn mean_anomaly(&self, mass: Num, epoch: Num) -> Num {
        let h = self.specific_angular_momentum(mass);
        let e = self.eccentricity;

        let epoch_diff = epoch - self.epoch;

        self.mean_anomaly_at_epoch + Self::mean_motion(h, e, mass) * epoch_diff
    }

    /// Mean motion
    /// https://en.wikipedia.org/wiki/Mean_anomaly
    pub fn mean_motion(h: Num, e: Num, mass: Num) -> Num {
        let μ = Self::standard_gravitational_parameter(mass);

        (μ.powi(2) / h.powi(3)) * (1.0 - e.powi(2)).powi(3).sqrt()
    }

    /// Hyperbolic mean anomaly
    /// SRC: https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-mean-anomaly
    pub fn hyperbolic_mean_anomaly(&self, mass: Num, epoch: Num) -> Num {
        let h = self.specific_angular_momentum(mass);
        let e = self.eccentricity;

        let epoch_diff = epoch - self.epoch;

        self.mean_anomaly_at_epoch + Self::hyperbolic_mean_motion(h, e, mass) * epoch_diff
    }

    /// Hyperbolic mean motion
    /// SRC: https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-mean-anomaly
    pub fn hyperbolic_mean_motion(h: Num, e: Num, mass: Num) -> Num {
        let μ = Self::standard_gravitational_parameter(mass);

        (μ.powi(2) / h.powi(3)) * (e.powi(2) - 1.0).powi(3).sqrt()
    }

    /// Eccentric Anomaly (E) is given by the equation:
    /// M = E - e * sin(E)
    /// where
    /// M is the mean anomaly
    /// e is the eccentricity
    ///
    /// https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/elliptical-orbits.html#equation-eq-keplers-equation-ellipse
    pub fn estimate_eccentric_anomaly(&self, mass: Num, epoch: Num, tolerance: Num) -> Num {
        let M = self.mean_anomaly(mass, epoch);
        let e = self.eccentricity;

        newton_approx(
            // f(E) = E - e*sin(E) - M
            |E| E - (e * E.sin()) - M,
            // f'(E) = 1 - e*cos(E)
            |E| 1.0 - (e * E.cos()),
            M,
            tolerance,
        )
    }

    /// Hyperbolic Anomaly (F) is given by the equation:
    /// M = e * sinh(F) - F
    /// where
    /// M is the mean anomaly
    /// e is the eccentricity
    ///
    /// https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-keplers-equation
    pub fn estimate_hyperbolic_anomaly(&self, mass: Num, epoch: Num, tolerance: Num) -> Num {
        let M = self.hyperbolic_mean_anomaly(mass, epoch);
        let e = self.eccentricity;

        newton_approx(
            // f(F) = e * sinh(F) - F - M
            |F| (e * F.sinh()) - F - M,
            // f'(F) = e * cosh(F) - 1
            |F| e * F.cosh() - 1.0,
            M,
            tolerance,
        )
    }

    pub fn state_vectors_at_epoch(&self, mass: Num, epoch: Num, tolerance: Num) -> StateVectors {
        // Lowercase nu
        println!();
        println!("Converting elements ({self:?}) to state vectors");
        let v = self.true_anomaly_at_epoch(mass, epoch, tolerance);

        StateVectors {
            position: self.position_at_true_anomaly(mass, v),
            velocity: self.velocity_at_true_anomaly(mass, v),
        }
    }

    #[inline]
    pub fn position_at_true_anomaly(&self, mass: Num, v: Num) -> Vec3 {
        let e = self.eccentricity;
        let h = self.specific_angular_momentum(mass);
        let μ = Self::standard_gravitational_parameter(mass);

        let r = (h.powi(2) / μ) / (1.0 + e * v.cos());

        // Perifocal coordinates
        let p = r * v.cos();
        let q = r * v.sin();

        let position = vec3(p, q, 0.0);

        self.perifocal_to_equatorial(position)
    }

    #[inline]
    pub fn velocity_at_true_anomaly(&self, mass: Num, v: Num) -> Vec3 {
        let e = self.eccentricity;
        let h = self.specific_angular_momentum(mass);
        let μ = Self::standard_gravitational_parameter(mass);

        let vp = -(μ / h) * v.sin();
        let vq = (μ / h) * (e + v.cos());

        self.perifocal_to_equatorial(vec3(vp, vq, 0.0))
    }

    #[inline(always)]
    pub fn perifocal_to_equatorial(&self, perifocal: Vec3) -> Vec3 {
        let mut m = Mat3::IDENTITY;

        let Ω = self.right_ascension_of_the_ascending_node;
        let i = self.inclination;
        let ω = self.argument_of_periapsis;

        m *= Mat3::from_rotation_z(Ω);
        m *= Mat3::from_rotation_x(i);
        m *= Mat3::from_rotation_z(ω);

        m.mul_vec3(perifocal)
    }

    pub fn specific_angular_momentum(&self, mass: Num) -> Num {
        let μ = Self::standard_gravitational_parameter(mass);
        let a = self.semi_major_axis;
        let e = self.eccentricity;

        // Derived from the equation for the semi-major-axis
        // https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/universal-variables.html#tab-ellipse-hyperbola-comparison
        if self.is_hyperbolic() {
            (μ * a * (e.powi(2) - 1.0)).sqrt()
        } else {
            (μ * a * (1.0 - e.powi(2))).sqrt()
        }
    }

    /// Calculates true anomaly
    pub fn true_anomaly_at_epoch(&self, mass: Num, epoch: Num, tolerance: Num) -> Num {
        let e = self.eccentricity;

        if self.is_hyperbolic() {
            let F = self.estimate_hyperbolic_anomaly(mass, epoch, tolerance);

            // https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-eccentric-anomaly-true-anomaly-hyperbola
            2.0 * ((F / 2.0).tanh() / ((e - 1.0) / (e + 1.0)).sqrt()).atan()
        } else {
            let E = self.estimate_eccentric_anomaly(mass, epoch, tolerance);

            // Circular (practically unattainable), elliptic or parabolic (practically unattainable)
            // https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/elliptical-orbits.html#equation-eq-eccentric-anomaly-true-anomaly-ellipse
            2.0 * ((E / 2.0).tan() / ((1.0 - e) / (1.0 + e)).sqrt()).atan()
        }
    }

    pub fn is_elliptical(&self) -> bool {
        self.eccentricity < 1.0
    }

    pub fn is_hyperbolic(&self) -> bool {
        // TODO: We ignore the parabolic case of e == 1.0
        self.eccentricity >= 1.0
    }
}
