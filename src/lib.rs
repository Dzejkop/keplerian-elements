#![allow(non_snake_case)]

use std::f32::consts::PI;

use glam::{vec3, Mat3, Vec3};

pub mod constants;
pub mod math;
pub mod utils;

use constants::G;
use math::newton_approx;
const TWO_PI: f32 = 2.0 * PI;

/// Data that defines a unique orbit in space
#[derive(Debug, Clone)]
pub struct KeplerianElements {
    pub eccentricity: f32,
    pub semi_major_axis: f32,
    pub inclination: f32,
    pub right_ascension_of_the_ascending_node: f32,
    pub argument_of_periapsis: f32,
    pub mean_anomaly_at_epoch: f32,
    pub epoch: f32,
}

/// Data that defines an orbit and position in space
pub struct Orbit {
    pub focus: Vec3,
    pub elements: KeplerianElements,
}

#[derive(Debug, Clone)]
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
    pub fn state_vectors_to_orbit(sv: StateVectors, mass: f32, time: f32) -> Self {
        // Position magnitude
        let rv = sv.position;
        let r = rv.length();
        let vv = sv.velocity;
        let v_mag = vv.length();

        // Radial velocity
        let vr = vv.dot(rv / r);

        // Azimuthal velocity
        let _vz = (v_mag.powi(2) - vr.powi(2)).sqrt();

        // Orbital angular momentum
        // This vector should point in the normal direction of the orbit
        let hv = rv.cross(vv);
        let h = hv.length();

        // Inclination
        // Equation is i = arccos(hz / h)
        let i = (hv.z / h).acos();

        // Right ascension of the ascending node

        // N vector and magnitude - it's the vector
        // parallel to the node line
        let nv = Vec3::Z.cross(hv);
        let n = nv.length();

        // We find the angle between the node line & the X axis
        let mut Ω = PI - (nv.x / n).acos();

        if nv.y < 0.0 {
            Ω = TWO_PI - Ω;
        }

        // Eccentricity
        let μ = Self::standard_gravitational_parameter(mass);
        let ev = (1.0 / μ) * ((v_mag.powi(2) - μ / r) * rv - r * vr * vv);

        let e = ev.length();

        let is_hyperbolic = e >= 1.0; // or parabolic

        // Argument of periapsis
        let ω = if e == 0.0 {
            // For a circular orbit the argument of periapsis is undefined
            0.0
        } else {
            let mut ω = PI - (ev.dot(nv) / (e * n)).acos();

            if ev.z < 0.0 {
                ω = TWO_PI - ω;
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
        let mut v = (rv / r).dot(ev / e).acos();

        if vr < 0.0 {
            v = TWO_PI - v;
        }

        // Mean anomaly
        let M = if is_hyperbolic {
            // https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-mean-anomaly-hyperbola
            (e * (e.powi(2) - 1.0).sqrt() * v.sin()) / (1.0 + e * v.cos())
                - (((e + 1.0).sqrt() + (e - 1.0).sqrt() * (v / 2.0).tan())
                    / ((e + 1.0).sqrt() - (e - 1.0).sqrt() * (v / 2.0).tan()))
                .ln()
        } else {
            // https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/elliptical-orbits.html#equation-eq-mean-anomaly-ellipse
            2.0 * (((1.0 - e) / (1.0 + e)).sqrt() * (v / 2.0).tan()).atan()
                - e * ((1.0 - e.powi(2)).sqrt() * v.sin() / (1.0 + e * v.cos()))
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

    pub fn ascending_node(&self) -> Vec3 {
        self.position_at_true_anomaly(PI + self.argument_of_periapsis)
    }

    pub fn descending_node(&self) -> Vec3 {
        self.position_at_true_anomaly(self.argument_of_periapsis)
    }

    pub fn periapsis(&self) -> Vec3 {
        self.position_at_true_anomaly(0.0)
    }

    pub fn apoapsis(&self) -> Vec3 {
        self.position_at_true_anomaly(PI)
    }

    pub fn normal(&self) -> Vec3 {
        self.perifocal_to_equatorial(Vec3::Z)
    }

    /// https://en.wikipedia.org/wiki/Standard_gravitational_parameter
    pub fn standard_gravitational_parameter(mass: f32) -> f32 {
        G * mass
    }

    /// https://en.wikipedia.org/wiki/Orbital_period
    pub fn period(&self, mass: f32) -> f32 {
        Self::period_static(self.semi_major_axis, mass)
    }

    pub fn period_static(a: f32, mass: f32) -> f32 {
        TWO_PI * (a.powi(3) / Self::standard_gravitational_parameter(mass)).sqrt()
    }

    /// https://en.wikipedia.org/wiki/Mean_anomaly
    pub fn mean_anomaly(&self, mass: f32, epoch: f32) -> f32 {
        let h = self.specific_angular_momentum(mass);
        let e = self.eccentricity;

        let epoch_diff = epoch - self.epoch;

        self.mean_anomaly_at_epoch + Self::mean_motion(h, e, mass) * epoch_diff
    }

    /// https://en.wikipedia.org/wiki/Mean_anomaly
    pub fn mean_motion(h: f32, e: f32, mass: f32) -> f32 {
        let μ = Self::standard_gravitational_parameter(mass);

        (μ.powi(2) / h.powi(3)) * (1.0 - e.powi(2)).powi(3).sqrt()
    }

    /// Hyperbolic mean anomaly
    /// SRC: https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-mean-anomaly
    pub fn hyperbolic_mean_anomaly(&self, mass: f32, epoch: f32) -> f32 {
        let h = self.specific_angular_momentum(mass);
        let e = self.eccentricity;

        let epoch_diff = epoch - self.epoch;

        self.mean_anomaly_at_epoch + Self::hyperbolic_mean_motion(h, e, mass) * epoch_diff
    }

    /// Hyperbolic mean anomaly
    /// SRC: https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-mean-anomaly
    pub fn hyperbolic_mean_motion(h: f32, e: f32, mass: f32) -> f32 {
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
    pub fn estimate_eccentric_anomaly(&self, mass: f32, epoch: f32, tolerance: f32) -> f32 {
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
    pub fn estimate_hyperbolic_anomaly(&self, mass: f32, epoch: f32, tolerance: f32) -> f32 {
        let M = self.hyperbolic_mean_anomaly(mass, epoch);
        let e = self.eccentricity;

        let x = newton_approx(
            // f(F) = e * sinh(F) - F - M
            |F| (e * F.sinh()) - F - M,
            // f'(F) = e * cosh(F) - 1
            |F| e * F.cosh() - 1.0,
            M,
            tolerance,
        );

        x
    }

    pub fn state_vectors_at_epoch(&self, mass: f32, epoch: f32, tolerance: f32) -> StateVectors {
        // Lowercase nu
        let v = self.true_anomaly_at_epoch(mass, epoch, tolerance);

        StateVectors {
            position: self.position_at_true_anomaly(v),
            velocity: self.velocity_at_true_anomaly(mass, v),
        }
    }

    pub fn position_at_true_anomaly(&self, v: f32) -> Vec3 {
        let a = self.semi_major_axis;
        let e = self.eccentricity;

        // Perifocal coordinates
        let (p, q) = if self.is_hyperbolic() {
            let r = (a * (e.powi(2) - 1.0)) / (1.0 + e * v.cos());

            let p = r * v.cos();
            let q = r * v.sin();

            (p, q)
        } else {
            let r = (a * (1.0 - e.powi(2))) / (1.0 + e * v.cos());

            let p = r * v.cos();
            let q = r * v.sin();

            (p, q)
        };

        let position = vec3(p, q, 0.0);

        self.perifocal_to_equatorial(position)
    }

    pub fn velocity_at_true_anomaly(&self, mass: f32, v: f32) -> Vec3 {
        let e = self.eccentricity;
        let h = self.specific_angular_momentum(mass);
        let μ = Self::standard_gravitational_parameter(mass);

        let v_mag = μ / h;

        let (vp, vq) = if self.is_hyperbolic() {
            (-v_mag * v.sin(), v_mag * (e + v.cos()))
        } else {
            (-v_mag * v.sin(), v_mag * (e + v.cos()))
        };

        let velocity = vec3(vp, vq, 0.0);

        self.perifocal_to_equatorial(velocity)
    }

    #[inline(always)]
    pub fn perifocal_to_equatorial(&self, perifocal: Vec3) -> Vec3 {
        let Ω = self.right_ascension_of_the_ascending_node;
        let i = self.inclination;
        let ω = self.argument_of_periapsis;

        let m = Mat3::from_rotation_z(-Ω) * (Mat3::from_rotation_x(-i) * Mat3::from_rotation_z(-ω));

        m.mul_vec3(perifocal)
    }

    pub fn specific_angular_momentum(&self, mass: f32) -> f32 {
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
    pub fn true_anomaly_at_epoch(&self, mass: f32, epoch: f32, tolerance: f32) -> f32 {
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
