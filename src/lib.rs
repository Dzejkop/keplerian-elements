#![allow(non_snake_case)]

#[cfg(feature = "f64")]
pub use glam::{dvec3 as vec3, DMat3 as Mat3, DVec3 as Vec3};
#[cfg(feature = "f32")]
pub use glam::{vec3, Mat3, Vec3};

#[cfg(feature = "f32")]
pub type Num = f32;

#[cfg(feature = "f64")]
pub type Num = f64;

pub mod astro;
pub mod constants;
pub mod math;
pub mod utils;

use constants::{G, PI};
use math::newton_approx;

const TWO_PI: Num = 2.0 * PI;

/// Data that defines a unique orbit in space
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

/// Data that defines an orbit and position in space
pub struct Orbit {
    pub focus: Vec3,
    pub elements: KeplerianElements,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StateVectors {
    pub position: Vec3,
    pub velocity: Vec3,
}

impl StateVectors {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self { position, velocity }
    }
}

impl KeplerianElements {
    pub fn state_vectors_to_orbit(sv: StateVectors, mass: Num, time: Num) -> Self {
        // Position magnitude
        let rv = sv.position;
        let r = rv.length();
        let vv = sv.velocity;
        let v_mag = vv.length();

        // Orbital angular momentum
        // This vector should point in the normal direction of the orbit
        let hv = rv.cross(vv);
        let h = hv.length();

        // N vector - it's the vector parallel to the node line
        let nv = Vec3::Z.cross(hv);
        let n = nv.length();

        // Eccentricity
        let μ = Self::standard_gravitational_parameter(mass);
        let ev = (1.0 / μ) * ((v_mag.powi(2) - (μ / r)) * rv - rv.dot(vv) * vv);
        let e = ev.length();

        let is_hyperbolic = e >= 1.0; // or parabolic

        // Right ascension of the ascending node

        // Inclination
        // Equation is i = arccos(hz / h)
        let i = (hv.z / h).acos();

        // We find the angle between the node line & the X axis
        let mut Ω = (nv.x / n).acos();

        if nv.y < 0.0 {
            Ω = TWO_PI - Ω;
        }

        if i.abs() < Num::EPSILON {
            Ω = 0.0;
        }

        // Argument of periapsis
        let ω = if e == 0.0 {
            // For a circular orbit the argument of periapsis is undefined
            0.0
        } else {
            let mut ω = (ev.dot(nv) / (e * n)).acos();

            if ev.z < 0.0 {
                ω = TWO_PI + ω;
            }

            ω
        };

        // Semi-major axis
        let a = if is_hyperbolic {
            (h.powi(2) / μ) / (e.powi(2) - 1.0)
        } else {
            (h.powi(2) / μ) / (1.0 - e.powi(2))
        };

        // True anomaly
        let mut v = (rv.dot(ev) / (r * e)).acos();

        if (rv.dot(vv)) < 0.0 {
            v = PI + v;
        }

        // Hyperbolic mean anomaly calculation
        fn calculate_hyperbolic_mean_anomaly(e: Num, v: Num) -> Num {
            let term1 = (e * (e.powi(2) - 1.0).sqrt() * v.sin()) / (1.0 + e * v.cos());
            let term2_numerator = (e + 1.0).sqrt() + (e - 1.0).sqrt() * (v / 2.0).tan();
            let term2_denominator = (e + 1.0).sqrt() - (e - 1.0).sqrt() * (v / 2.0).tan();

            term1 - (term2_numerator / term2_denominator).ln()
        }

        // Elliptical mean anomaly calculation
        fn calculate_elliptical_mean_anomaly(e: Num, v: Num) -> Num {
            let term1 = 2.0 * (((1.0 - e) / (1.0 + e)).sqrt() * (v / 2.0).tan()).atan();
            let term2 = e * ((1.0 - e.powi(2)).sqrt() * v.sin() / (1.0 + e * v.cos()));

            term1 - term2
        }

        // Mean anomaly calculation
        let M = if is_hyperbolic {
            calculate_hyperbolic_mean_anomaly(e, v)
        } else {
            calculate_elliptical_mean_anomaly(e, v)
        };

        Self {
            eccentricity: e,
            semi_major_axis: a,
            inclination: i,
            right_ascension_of_the_ascending_node: Ω,
            argument_of_periapsis: ω,
            mean_anomaly_at_epoch: M,
            epoch: time,
        }
    }

    pub fn ascending_node(&self, mass: f32) -> Vec3 {
        self.position_at_true_anomaly(mass, self.argument_of_periapsis)
    }

    pub fn descending_node(&self, mass: f32) -> Vec3 {
        self.position_at_true_anomaly(mass, PI + self.argument_of_periapsis)
    }

    pub fn periapsis(&self, mass: f32) -> Vec3 {
        self.position_at_true_anomaly(mass, 0.0)
    }

    pub fn apoapsis(&self, mass: f32) -> Vec3 {
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
        let v = self.true_anomaly_at_epoch(mass, epoch, tolerance);

        StateVectors {
            position: self.position_at_true_anomaly(mass, v),
            velocity: self.velocity_at_true_anomaly(mass, v),
        }
    }

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

        // let m = Mat3::from_rotation_z(-Ω) * (Mat3::from_rotation_x(-i) * Mat3::from_rotation_z(-ω));

        m *= Mat3::from_rotation_z(-Ω);
        m *= Mat3::from_rotation_x(-i);
        m *= Mat3::from_rotation_z(-ω);

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

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    const MASS: f32 = 100_000_000_000.0;
    const EPOCH: f32 = 0.0;
    const MAX_ABS_DIFF: f32 = 0.0001;
    const TOLERANCE: f32 = 0.0001;

    #[test]
    fn conversion() {
        let position = vec3(125.0, 0.0, 1.0);
        let velocity = vec3(0.0000000001, 0.0001, 0.0000000001);

        let sv = StateVectors::new(position, velocity);
        let elements = KeplerianElements::state_vectors_to_orbit(sv, MASS, EPOCH);
        println!("Elements: {elements:#?}");

        let sv_converted = elements.state_vectors_at_epoch(MASS, EPOCH, TOLERANCE);
        println!("State vectors converted: {sv_converted:#?}");

        let elements_converted =
            KeplerianElements::state_vectors_to_orbit(sv_converted, MASS, EPOCH);
        println!("Elements converted: {elements_converted:#?}");

        assert!(
            sv.position.abs_diff_eq(sv_converted.position, MAX_ABS_DIFF),
            "Position {:?} not equal {:?}",
            sv.position,
            sv_converted.position
        );
        assert!(
            sv.velocity.abs_diff_eq(sv_converted.velocity, MAX_ABS_DIFF),
            "Velocity {:?} not equal {:?}",
            sv.velocity,
            sv_converted.velocity
        );
    }

    #[test_case(0.0, vec3(0.0, 1.0, 0.0))]
    #[test_case(PI / 2.0, vec3(1.0, 0.0, 0.0))]
    #[test_case(PI, vec3(0.0, -1.0, 0.0))]
    #[test_case(PI + (PI / 2.0), vec3(-1.0, 0.0, 0.0))]
    fn elements_to_position(v: f32, exp: Vec3) {
        let elements = KeplerianElements {
            eccentricity: 0.0,
            semi_major_axis: 1.0,
            inclination: 0.0,
            right_ascension_of_the_ascending_node: 0.0,
            argument_of_periapsis: 0.0,
            mean_anomaly_at_epoch: 0.0,
            epoch: 0.0,
        };

        let position = elements.position_at_true_anomaly(MASS, v);
        let velocity = elements.velocity_at_true_anomaly(MASS, v);

        println!("velocity = {velocity:#?}");

        assert!(
            position.abs_diff_eq(exp, MAX_ABS_DIFF),
            "Position {:?} not equal {:?}",
            position,
            exp
        );
    }
}
