#![allow(non_snake_case)]

use std::f32::consts::{FRAC_PI_2, PI};

use glam::{vec3, Mat3, Quat, Vec2, Vec3};

pub mod constants;

use constants::G;

/// Data that defines a unique orbit in space
#[derive(Debug, Clone)]
pub struct KeplerianElements {
    pub eccentricity: f32,
    pub semi_major_axis: f32,
    pub inclination: f32,
    pub longitude_of_ascending_node: f32,
    pub argument_of_periapsis: f32,
    pub mean_anomaly_at_epoch_zero: f32,
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
        let vv = sv.velocity;
        let r = rv.length();

        // Orbital angular momentum
        let hv = rv.cross(vv);
        let h = hv.length();

        // Inclination
        // Equation is i = arccos(hz / h)
        // but we're in a Y-up coordinate system
        let i = (hv.y / h).acos();

        // Right ascension of the ascending node
        let nv = Vec3::Y.cross(hv);
        let n = nv.length();

        let Ω = PI - (nv.x / n).acos();

        // Eccentricity
        let μ = Self::standard_gravitational_parameter(mass);
        let ev = vv.cross(hv) / μ - rv / r;
        let e = ev.length();

        // Argument of periapsis
        // let ω = FRAC_PI_2 - (nv.dot(ev) / n * e).acos();
        let ω = (nv.dot(ev) / n * e).acos();
        // let ω = if ev.y >= 0.0 { ω } else { 2.0 * PI - ω };

        let ω = FRAC_PI_2 - ω;

        // True anomaly
        let v = (rv / r).dot(ev / e).acos();

        // Semi-major axis
        let ϵ = (v.powi(2) / 2.0) - (μ / r); // Specific orbital energy
        let a = -μ / (2.0 * ϵ);
        // let a = r * (1.0 + e * v.cos()) / (1.0 - e.powi(2));

        // Mean anomaly
        let t1 = -(1.0 - e.powi(2)).sqrt() * v.sin();
        let t2 = -e - v.cos();
        let t3 = 1.0 + e * v.cos();

        let M = f32::atan2(t1, t2) + PI - e * (t1 / t3);

        let mean_motion = Self::mean_motion_static(a, mass);
        let mean_anomaly_at_epoch_zero = M - mean_motion * time;

        Self {
            eccentricity: e,
            semi_major_axis: a,
            inclination: i,
            longitude_of_ascending_node: Ω,
            argument_of_periapsis: ω,
            mean_anomaly_at_epoch_zero,
        }
    }

    pub fn ascending_node(&self) -> Vec3 {
        self.position_at_true_anomaly(-self.argument_of_periapsis)
    }

    pub fn periapsis(&self) -> Vec3 {
        self.position_at_true_anomaly(0.0)
    }

    pub fn apoapsis(&self) -> Vec3 {
        self.position_at_true_anomaly(PI)
    }

    pub fn normal(&self) -> Vec3 {
        self.perifocal_to_equatorial(Vec3::Y)
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
        2.0 * PI * (a.powi(3) / Self::standard_gravitational_parameter(mass)).sqrt()
    }

    /// https://en.wikipedia.org/wiki/Mean_motion
    pub fn mean_motion(&self, mass: f32) -> f32 {
        Self::mean_motion_static(self.semi_major_axis, mass)
    }

    pub fn mean_motion_static(a: f32, mass: f32) -> f32 {
        let μ = Self::standard_gravitational_parameter(mass);
        (μ / a.powi(3)).sqrt()
    }

    /// https://en.wikipedia.org/wiki/Mean_anomaly
    pub fn mean_anomaly(&self, mass: f32, epoch: f32) -> f32 {
        self.mean_anomaly_at_epoch_zero + self.mean_motion(mass) * epoch
    }

    /// https://en.wikipedia.org/wiki/Eccentric_anomaly
    ///
    /// Eccentric Anomaly (E) is given by the equation:
    /// M = E - e * sin(E)
    /// where
    /// M is the mean anomaly
    /// e is the eccentricity
    ///
    /// To estimate the Eccentric Anomaly we use Newton's method
    /// https://en.wikipedia.org/wiki/Newton%27s_method
    #[allow(non_snake_case)]
    pub fn estimate_eccentric_anomaly(&self, mass: f32, epoch: f32, tolerance: f32) -> f32 {
        let M = self.mean_anomaly(mass, epoch);
        let mut E = M;
        let e = self.eccentricity;

        let mut error = 1.0;

        while error > tolerance {
            // f(E) = E - e*sin(E) - M
            let fe = E - (e * E.sin()) - M;

            // f'(E) = 1 - e*cos(E)
            let fe_prime = 1.0 - (e * E.cos());

            let next_E = E - (fe / fe_prime);

            error = (next_E - E).abs();
            E = next_E;
        }

        E
    }

    /// https://en.wikipedia.org/wiki/Eccentric_anomaly
    ///
    /// Hyperbolic Anomaly (H) is given by the equation:
    /// M = e * sinh(H) - H
    /// where
    /// M is the mean anomaly
    /// e is the eccentricity
    ///
    /// To estimate the Hyperbolic Anomaly we use Newton's method
    /// https://en.wikipedia.org/wiki/Newton%27s_method
    #[allow(non_snake_case)]
    pub fn estimate_hyperbolic_anomaly(&self, mass: f32, epoch: f32, tolerance: f32) -> f32 {
        let M = self.mean_anomaly(mass, epoch);
        let mut H = M;
        let e = self.eccentricity;

        let mut error = 1.0;

        while error > tolerance {
            // f(H) = e * sinh(H) - H - M
            let fe = H - (e * H.sin()) - M;

            // f'(H) = e * cosh(H) - 1
            let fe_prime = e * H.cosh() - 1.0;

            let next_H = H - (fe / fe_prime);

            error = (next_H - H).abs();
            H = next_H;
        }

        H
    }

    #[allow(non_snake_case)]
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

        let r = (a * (1.0 - e.powi(2))) / (1.0 + e * v.cos());

        // Perifocal coordinates
        let x = r * v.cos();
        let y = r * v.sin();

        // Y-up coordinate system
        let p = vec3(x, 0.0, y);

        self.perifocal_to_equatorial(p)
    }

    pub fn velocity_at_true_anomaly(&self, mass: f32, v: f32) -> Vec3 {
        let e = self.eccentricity;
        let h = self.specific_angular_momentum(mass);
        let μ = Self::standard_gravitational_parameter(mass);

        let v_mag = μ / h;

        let vx = -v_mag * v.sin();
        let vy = v_mag * (e + v.cos());

        // Y-up coordinate system
        let velocity = vec3(vx, 0.0, vy);

        self.perifocal_to_equatorial(velocity)
    }

    #[inline(always)]
    pub fn perifocal_to_equatorial(&self, perifocal: Vec3) -> Vec3 {
        let Ω = self.longitude_of_ascending_node;
        let i = self.inclination;
        let ω = self.argument_of_periapsis;

        // Compute rotation matrices to transform perifocal frame to ecliptic frame
        let rot_Ω = Mat3::from_axis_angle(-Vec3::Y, Ω);
        let rot_i = Mat3::from_axis_angle(Vec3::X, i);
        let rot_ω = Mat3::from_axis_angle(-Vec3::Y, ω);

        // Compute rotation quaternion
        let q = Quat::from_mat3(&(rot_Ω * rot_i * rot_ω));

        q.mul_vec3(perifocal)
    }

    pub fn specific_angular_momentum(&self, mass: f32) -> f32 {
        let μ = Self::standard_gravitational_parameter(mass);
        let a = self.semi_major_axis;
        let e = self.eccentricity;

        f32::sqrt(μ * a * (1.0 - e.powi(2)))
    }

    /// Calculates true anomaly
    #[allow(non_snake_case)]
    pub fn true_anomaly_at_epoch(&self, mass: f32, epoch: f32, tolerance: f32) -> f32 {
        let e = self.eccentricity;

        if e <= 1.0 {
            let E = self.estimate_eccentric_anomaly(mass, epoch, tolerance);

            // Circular (practically unattainable), elliptic or parabolic (practically unattainable)
            let one_plus_e = 1.0 + e;
            let one_minus_e = 1.0 - e;

            let term_1 = f32::sqrt(one_plus_e / one_minus_e);
            let term_2 = f32::tan(E / 2.0);

            2.0 * f32::atan(term_1 * term_2)
        } else {
            // Hyperbolic
            let H = self.estimate_hyperbolic_anomaly(mass, epoch, tolerance);

            let e_plus_one = e + 1.0;
            let e_minus_one = e - 1.0;

            let term_1 = f32::sqrt(e_plus_one / e_minus_one);
            let term_2 = f32::tanh(H / 2.0);

            2.0 * f32::atan(term_1 * term_2)
        }
    }
}
