use crate::astro::{self, standard_gravitational_parameter};
use crate::{vec3, Mat3, Num, StateVectors, Vec3, PI, TWO_PI};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        diff +=
            (self.argument_of_periapsis - other.argument_of_periapsis).abs();
        diff +=
            (self.mean_anomaly_at_epoch - other.mean_anomaly_at_epoch).abs();

        diff
    }

    pub fn from_state_vectors(
        state_vectors: &StateVectors,
        mass: Num,
        time: Num,
    ) -> Self {
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

    /// https://en.wikipedia.org/wiki/Orbital_period
    pub fn period(&self, mass: Num) -> Num {
        astro::period(self.semi_major_axis, mass)
    }

    /// https://en.wikipedia.org/wiki/Mean_anomaly
    pub fn mean_anomaly(&self, mass: Num, epoch: Num) -> Num {
        let h = self.specific_angular_momentum(mass);
        let e = self.eccentricity;

        let epoch_diff = epoch - self.epoch;

        self.mean_anomaly_at_epoch
            + astro::elliptic::mean_motion(h, e, mass) * epoch_diff
    }

    /// Hyperbolic mean anomaly
    /// SRC: https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-mean-anomaly
    pub fn hyperbolic_mean_anomaly(&self, mass: Num, epoch: Num) -> Num {
        let h = self.specific_angular_momentum(mass);
        let e = self.eccentricity;

        let epoch_diff = epoch - self.epoch;

        self.mean_anomaly_at_epoch
            + astro::hyperbolic::mean_motion(h, e, mass) * epoch_diff
    }

    /// Eccentric Anomaly (E) is given by the equation:
    /// M = E - e * sin(E)
    /// where
    /// M is the mean anomaly
    /// e is the eccentricity
    ///
    /// https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/elliptical-orbits.html#equation-eq-keplers-equation-ellipse
    pub fn estimate_eccentric_anomaly(
        &self,
        mass: Num,
        epoch: Num,
        tolerance: Num,
    ) -> Num {
        let M = self.mean_anomaly(mass, epoch);
        let e = self.eccentricity;

        astro::elliptic::estimate_anomaly(M, e, tolerance)
    }

    /// Hyperbolic Anomaly (F) is given by the equation:
    /// M = e * sinh(F) - F
    /// where
    /// M is the mean anomaly
    /// e is the eccentricity
    ///
    /// https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-keplers-equation
    pub fn estimate_hyperbolic_anomaly(
        &self,
        mass: Num,
        epoch: Num,
        tolerance: Num,
    ) -> Num {
        let M = self.hyperbolic_mean_anomaly(mass, epoch);
        let e = self.eccentricity;

        astro::hyperbolic::estimate_anomaly(M, e, tolerance)
    }

    pub fn state_vectors_at_epoch(
        &self,
        mass: Num,
        epoch: Num,
        tolerance: Num,
    ) -> StateVectors {
        // Lowercase nu
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
        let μ = standard_gravitational_parameter(mass);

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
        let μ = standard_gravitational_parameter(mass);

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
        let μ = standard_gravitational_parameter(mass);
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
    pub fn true_anomaly_at_epoch(
        &self,
        mass: Num,
        epoch: Num,
        tolerance: Num,
    ) -> Num {
        let e = self.eccentricity;

        if self.is_hyperbolic() {
            let F = self.estimate_hyperbolic_anomaly(mass, epoch, tolerance);
            astro::hyperbolic::true_anomaly(F, e)
        } else {
            let E = self.estimate_eccentric_anomaly(mass, epoch, tolerance);
            astro::elliptic::true_anomaly(E, e)
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
