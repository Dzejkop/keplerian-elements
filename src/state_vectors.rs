use crate::astro::{self, standard_gravitational_parameter};
use crate::{KeplerianElements, Num, Vec3, TWO_PI};

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateVectors {
    pub position: Vec3,
    pub velocity: Vec3,
}

impl StateVectors {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self { position, velocity }
    }

    pub fn abs_diff(&self, other: &Self) -> Num {
        self.position.distance(other.position)
            + self.velocity.distance(other.velocity)
    }

    pub fn to_elements(&self, mass: Num, time: Num) -> KeplerianElements {
        // Position magnitude
        let rv = self.position;
        let r = rv.length();
        let vv = self.velocity;
        let v_mag = vv.length();

        // Orbital angular momentum
        // This vector should point in the normal direction of the orbit
        let hv = rv.cross(vv);
        let h = hv.length();

        // N vector - vector that lies on the node line in the direction of the ascending node
        let mut nv = Vec3::Z.cross(hv);

        // If inclinations is 0 this vector will be zero as well
        // since a lot of other arguments depend on this vector
        // we set it to the X axis
        if nv.length() < Num::EPSILON {
            nv = Vec3::X;
        }

        let nv = nv.normalize();

        // Eccentricity
        let μ = standard_gravitational_parameter(mass);

        let ev = (1.0 / μ) * ((v_mag.powi(2) - (μ / r)) * rv - rv.dot(vv) * vv);
        let e = ev.length();

        let is_hyperbolic = e >= 1.0; // or parabolic

        // Right ascension of the ascending node

        // Inclination
        // Equation is i = arccos(hz / h)
        let i = (hv.z / h).acos();

        // We find the angle between the node line & the X axis
        let mut Ω = (nv.x).acos();

        if nv.y < 0.0 {
            Ω = TWO_PI - Ω;
        }

        if i.abs() < Num::EPSILON {
            Ω = 0.0;
        }

        // Argument of periapsis
        let mut ω = (ev / e).dot(nv).acos();

        // An edge case for a zero inclination orbit
        // If an orbit has zero inclination,
        // the z component of the eccentricity vector
        // is zero.
        //
        // But we can still do a quadrant check using the y component
        if i.abs() < Num::EPSILON {
            if ev.y < 0.0 {
                ω = TWO_PI - ω;
            }
        } else {
            if ev.z < 0.0 {
                ω = TWO_PI - ω;
            }
        }

        if e == 0.0 {
            // For a circular orbit the argument of periapsis is undefined
            ω = 0.0
        }

        // Semi-major axis
        let a = if is_hyperbolic {
            (h.powi(2) / μ) / (e.powi(2) - 1.0)
        } else {
            (h.powi(2) / μ) / (1.0 - e.powi(2))
        };

        // True anomaly
        let mut v = (rv / r).dot(ev / e).acos();

        if ((rv / r).dot(vv / v_mag)) < 0.0 {
            v = TWO_PI - v;
        }

        // Mean anomaly calculation
        let M = if is_hyperbolic {
            calculate_hyperbolic_mean_anomaly(e, v)
        } else {
            calculate_elliptical_mean_anomaly(e, v)
        };

        KeplerianElements {
            eccentricity: e,
            semi_major_axis: a,
            inclination: i,
            right_ascension_of_the_ascending_node: Ω,
            argument_of_periapsis: ω,
            mean_anomaly_at_epoch: M,
            epoch: time,
        }
    }

    pub fn semi_major_axis(&self, central_mass: Num) -> Num {
        let μ = standard_gravitational_parameter(central_mass);
        let r = self.position.length();
        let v = self.velocity.length();

        μ * r / (r * v.powi(2) - 2.0 * μ)
    }

    pub fn period(&self, central_mass: Num) -> Num {
        let μ = standard_gravitational_parameter(central_mass);

        let r0 = self.position.length();
        let v0 = self.velocity.length();
        let ξ = 0.5 * v0 * v0 - μ / r0;
        let a = -μ / (2.0 * ξ);

        astro::period(a, central_mass)
    }

    pub fn propagate_kepler(
        self,
        dt: Num,
        mass: Num,
        tolerance: Num,
    ) -> StateVectors {
        let μ = standard_gravitational_parameter(mass);

        let r0 = self.position.length();
        let v0 = self.velocity.length();
        let ξ = 0.5 * v0 * v0 - μ / r0;
        let a = -μ / (2.0 * ξ);
        let one_over_a = 1.0 / a;

        let x0 = if one_over_a > 0.000001 {
            μ.sqrt() * dt * one_over_a
        } else if one_over_a < -0.000001 {
            dt.signum()
                * (-a).sqrt()
                * ((-2.0 * μ * one_over_a * dt)
                    / (self.position.dot(self.velocity)
                        + dt.signum()
                            * (-μ * a).sqrt()
                            * (1.0 - r0 * one_over_a)))
                    .ln()
        } else {
            let h = self.position.cross(self.velocity);
            let p = h.length().powi(2) / μ;
            let s = 0.5 * (1.0 / (3.0 * (μ / (p * p * p)).sqrt() * dt)).atan();
            let w = (s.tan().powf(1.0 / 3.0)).atan();
            p.sqrt() * 2.0 / (2.0 * w).tan()
        };

        let mut x = x0;
        let mut r: Num = 0.0;
        let (mut c2, mut c3);
        c2 = 0.0;
        c3 = 0.0;
        let mut ψ: Num = 0.0;

        for _ in 0..500 {
            ψ = x * x * one_over_a;
            let result = find_c2c3(ψ, tolerance);
            c2 = result.0;
            c3 = result.1;

            r = x * x * c2
                + self.position.dot(self.velocity) / μ.sqrt()
                    * x
                    * (1.0 - ψ * c3)
                + r0 * (1.0 - ψ * c2);
            let new_x = x
                + (μ.sqrt() * dt
                    - x * x * x * c3
                    - self.position.dot(self.velocity) / μ.sqrt() * x * x * c2
                    - r0 * x * (1.0 - ψ * c3))
                    / r;

            if (new_x - x).abs() < tolerance {
                break;
            }
            x = new_x;
        }

        let f = 1.0 - x.powi(2) / r0 * c2;
        let g = dt - x.powi(3) / μ.sqrt() * c3;
        let dg = 1.0 - x.powi(2) / r * c2;
        let df = μ.sqrt() / (r * r0) * x * (ψ * c3 - 1.0);

        let new_position = self.position * f + self.velocity * g;
        let new_velocity = self.position * df + self.velocity * dg;

        let ret = StateVectors {
            position: new_position,
            velocity: new_velocity,
        };

        if ret.position.is_nan() || ret.velocity.is_nan() {
            eprintln!("propagate_kepler({self:?}, {dt}, {mass}, {tolerance}) -> {ret:?}");
            panic!("Kepler propagation failed");
        }

        ret
    }
}

// Hyperbolic mean anomaly calculation
fn calculate_hyperbolic_mean_anomaly(e: Num, v: Num) -> Num {
    let term1 = (e * (e.powi(2) - 1.0).sqrt() * v.sin()) / (1.0 + e * v.cos());
    let term2_numerator = (e + 1.0).sqrt() + (e - 1.0).sqrt() * (v / 2.0).tan();
    let term2_denominator =
        (e + 1.0).sqrt() - (e - 1.0).sqrt() * (v / 2.0).tan();

    term1 - (term2_numerator / term2_denominator).ln()
}

// Elliptical mean anomaly calculation
fn calculate_elliptical_mean_anomaly(e: Num, v: Num) -> Num {
    let term1 = 2.0 * (((1.0 - e) / (1.0 + e)).sqrt() * (v / 2.0).tan()).atan();
    let term2 = e * ((1.0 - e.powi(2)).sqrt() * v.sin() / (1.0 + e * v.cos()));

    term1 - term2
}

fn find_c2c3(ψ: Num, tolerance: Num) -> (Num, Num) {
    if ψ > tolerance {
        let sqrt_ψ = ψ.sqrt();
        (
            (1.0 - sqrt_ψ.cos()) / ψ,
            (sqrt_ψ - sqrt_ψ.sin()) / ψ.powi(3).sqrt(),
        )
    } else if ψ < -tolerance {
        let sqrt_ψ = (-ψ).sqrt();
        (
            (1.0 - sqrt_ψ.cosh()) / ψ,
            (sqrt_ψ.sinh() - sqrt_ψ) / (-ψ).powi(3).sqrt(),
        )
    } else {
        (0.5, 1.0 / 6.0)
    }
}
