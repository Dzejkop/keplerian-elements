#![allow(non_snake_case)]

use std::f32::consts::PI;

use glam::{Mat3, Quat, Vec3};

pub struct Orbit {
    pub eccentricity: f32,
    pub semi_major_axis: f32,
    pub inclination: f32,
    pub longitude_of_ascending_node: f32,
    pub argument_of_periapsis: f32,
}

pub struct StateVectors {
    pub position: Vec3,
    pub velocity: Vec3,
}

impl StateVectors {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self { position, velocity }
    }
}

/// Gravitational constant
const G: f32 = 6.67430e-11;

impl Orbit {
    pub fn state_vectors_to_orbit(state_vectors: StateVectors, central_body_mass: f32) -> Self {
        // Compute the magnitude of the position vector and velocity vector
        let r_mag = state_vectors.position.length();
        let v_mag = state_vectors.velocity.length();

        // Compute the specific angular momentum vector
        let h = state_vectors.position.cross(state_vectors.velocity);
        let h_mag = h.length();

        // Compute the eccentricity vector
        let e = (state_vectors.velocity.cross(h) / central_body_mass)
            - (state_vectors.position / r_mag);
        let e_mag = e.length();

        // Compute the semi-major axis
        let a = 1.0 / (2.0 / r_mag - v_mag * v_mag / (central_body_mass * central_body_mass));

        // Compute the inclination
        let i = h.z / h_mag;

        // Compute the longitude of ascending node
        let n = Vec3::new(-h.y, h.x, 0.0).normalize();
        let o = n.cross(Vec3::Z);
        let sign = o.z.signum();
        let cos_omega = n.dot(Vec3::X) / o.length();
        let sin_omega = sign * o.y / o.length();
        let omega = sin_omega.atan2(cos_omega);

        // Compute the argument of periapsis
        let cos_w = e.dot(n) / (e_mag * n.length());
        let sin_w = e.dot(o) / (e_mag * o.length());
        let w = sin_w.atan2(cos_w);

        // Create a new Orbit object with the computed Keplerian elements
        Orbit {
            eccentricity: e_mag,
            semi_major_axis: a,
            inclination: i.asin(),
            longitude_of_ascending_node: omega,
            argument_of_periapsis: w,
        }
    }

    /// https://en.wikipedia.org/wiki/Standard_gravitational_parameter
    pub fn standard_gravitational_parameter(mass: f32) -> f32 {
        G * mass
    }

    /// https://en.wikipedia.org/wiki/Orbital_period
    pub fn period(&self, mass: f32) -> f32 {
        let sm_cubed = self.semi_major_axis * self.semi_major_axis * self.semi_major_axis;

        2.0 * PI * (sm_cubed / Self::standard_gravitational_parameter(mass)).sqrt()
    }

    /// https://en.wikipedia.org/wiki/Mean_motion
    pub fn mean_motion(&self, mass: f32) -> f32 {
        2.0 * PI / self.period(mass)
    }

    /// https://en.wikipedia.org/wiki/Mean_anomaly
    pub fn mean_anomaly(&self, mass: f32, time: f32) -> f32 {
        self.mean_motion(mass) * time
    }

    /// https://en.wikipedia.org/wiki/Eccentric_anomaly
    ///
    /// Eccentric Anomaly (EA) is given by the equation:
    /// M = EA - e*sin(EA)
    /// where
    /// M is the mean anomaly
    /// e is the eccentricity
    ///
    /// To estimate the Eccentric Anomaly we use Newton's method
    /// https://en.wikipedia.org/wiki/Newton%27s_method
    pub fn estimate_eccentric_anomaly(&self, mass: f32, time: f32, tolerance: f32) -> f32 {
        let mean_anomaly = self.mean_anomaly(mass, time);
        let mut eccentric_anomaly = mean_anomaly;

        let mut error = 1.0;

        while error > tolerance {
            // f(E) = E - e*sin(E) - M
            let fe =
                eccentric_anomaly - (self.eccentricity * eccentric_anomaly.sin()) - mean_anomaly;

            // f'(E) = 1 - e*cos(E)
            let fe_prime = 1.0 - (self.eccentricity * eccentric_anomaly.cos());

            let next_eccentric_anomaly = eccentric_anomaly - (fe / fe_prime);

            error = (next_eccentric_anomaly - eccentric_anomaly).abs();
            eccentric_anomaly = next_eccentric_anomaly;
        }

        eccentric_anomaly
    }

    pub fn state_vectors_at_epoch(&self, mass: f32, time: f32, tolerance: f32) -> StateVectors {
        let Ω = self.longitude_of_ascending_node;
        let ω = self.argument_of_periapsis;
        let i = self.inclination;
        let a = self.semi_major_axis;
        let e = self.eccentricity;
        let μ = Self::standard_gravitational_parameter(mass);

        let E = self.estimate_eccentric_anomaly(mass, time, tolerance);

        // Specific angular momentum
        let h = f32::sqrt(μ * a * (1.0 - e.powi(2)));

        // Perifocal x and y
        let x = a * (f32::cos(E) - e);
        let y = a * f32::sqrt(1.0 - e.powi(2)) * f32::sin(E);

        // Perifocal velocity
        let vx = -f32::sin(E) * h / (a * (1.0 - e * f32::cos(E)));
        let vy = f32::sqrt(1.0 - e.powi(2)) * f32::cos(E) * h / (a * (1.0 - e * f32::cos(E)));

        // Vectors
        let position_perifocal = Vec3::new(x, 0.0, y);
        let velocity_perifocal = Vec3::new(vx, 0.0, vy);

        // Compute rotation matrices to transform perifocal frame to ecliptic frame
        let i = Mat3::from_axis_angle(Vec3::X, i);
        let o = Mat3::from_axis_angle(Vec3::Z, Ω);
        let w = Mat3::from_axis_angle(Vec3::Z, ω);

        // Compute rotation quaternion
        let q = Quat::from_mat3(&(o * i * w));

        let position = q.mul_vec3(position_perifocal);
        let velocity = q.mul_vec3(velocity_perifocal);

        StateVectors { position, velocity }
    }
}
