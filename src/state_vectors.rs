use crate::astro::standard_gravitational_parameter;
use crate::{KeplerianElements, Num, Vec3, TWO_PI};

#[derive(Debug, Default, Clone, Copy)]
pub struct StateVectors {
    pub position: Vec3,
    pub velocity: Vec3,
}

impl StateVectors {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self { position, velocity }
    }

    pub fn abs_diff(&self, other: &Self) -> Num {
        self.position.distance(other.position) + self.velocity.distance(other.velocity)
    }

    pub fn to_elements(&self, mass: Num, time: Num) -> KeplerianElements {
        println!();
        println!("Calculating elements from state vectors");
        println!("self = {self:?}");
        println!("mass = {mass:?}");
        println!("time = {time:?}");

        // Position magnitude
        let rv = self.position;
        let r = rv.length();
        let vv = self.velocity;
        let v_mag = vv.length();

        println!("rv = {rv:?}");
        println!("r = {r}");
        println!("vv = {vv:?}");
        println!("v_mag = {v_mag}");

        // Orbital angular momentum
        // This vector should point in the normal direction of the orbit
        let hv = rv.cross(vv);
        let h = hv.length();

        println!("hv = {hv:?}");
        println!("h = {h}");

        // N vector - it's the vector parallel to the node line
        // println!("Z.dot(hv) = {}", Vec3::Z.dot(hv));
        // println!("hv.dot(Z) = {}", hv.dot(Vec3::Z));

        // let nv = if hv.angle_between(Vec3::Z) < 0.1 {
        //     // Arbitrary vector perpendicular to Z
        //     Vec3::X * r
        // } else {
        //     Vec3::Z.cross(hv)
        // };

        let mut nv = Vec3::Z.cross(hv);
        let mut n = nv.length();

        if nv.length() < Num::EPSILON {
            // Arbitrary vector perpendicular to Z
            println!("Correctinv NV");
            nv = Vec3::X * r;
            n = nv.length();
        }

        println!("nv = {nv:?}");
        println!("n = {n}");

        // Eccentricity
        let μ = standard_gravitational_parameter(mass);

        let ev = (1.0 / μ) * ((v_mag.powi(2) - (μ / r)) * rv - rv.dot(vv) * vv);
        let e = ev.length();

        println!("ev = {ev:?}");
        println!("e = {e}");

        let is_hyperbolic = e >= 1.0; // or parabolic

        // Right ascension of the ascending node

        // Inclination
        // Equation is i = arccos(hz / h)
        let i = (hv.z / h).acos();
        println!("i = {i}");

        // We find the angle between the node line & the X axis
        let mut Ω = (nv.x / n).acos();

        if nv.y < 0.0 {
            Ω = TWO_PI - Ω;
        }

        if i.abs() < Num::EPSILON {
            Ω = 0.0;
        }

        println!("Ω = {Ω}");

        // Argument of periapsis
        println!("ev.dot(nv) / (e * n) = {}", ev.dot(nv) / (e * n));
        println!("(ev / e).dot(nv / n) = {}", (ev / e).dot(nv / n));
        let mut ω = (ev / e).dot(nv / n).acos();

        if ev.z < 0.0 {
            println!("Adjusting ω");
            ω = TWO_PI - ω;
        }

        if e == 0.0 {
            // For a circular orbit the argument of periapsis is undefined
            ω = 0.0
        }

        println!("ω = {ω}");

        // Semi-major axis
        let a = if is_hyperbolic {
            (h.powi(2) / μ) / (e.powi(2) - 1.0)
        } else {
            (h.powi(2) / μ) / (1.0 - e.powi(2))
        };

        println!("a = {a}");

        // True anomaly
        let mut v = (rv / r).dot(ev / e).acos();

        println!("(rv / r).dot(vv / v_mag) = {:?}", (rv / r).dot(vv / v_mag));
        if ((rv / r).dot(vv / v_mag)) < 0.0 {
            println!("Adjusting v");
            v = TWO_PI - v;
        }

        println!("v = {v}");

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3;

    const MASS: Num = 19890000.0;

    #[test]
    fn incremental_difference() {
        let a = StateVectors {
            position: vec3(-661207000.0, 348865760.0, 13339082.0),
            velocity: vec3(-6.25e-7, -1.1828758e-6, 1.9684657e-8),
        };
        let b = StateVectors {
            position: vec3(-661207000.0, 348865760.0, 13339082.0),
            velocity: vec3(-6.1500003e-7, -1.1828758e-6, 1.9684657e-8),
        };

        let orbit_a = a.to_elements(MASS, 0.0);
        println!();
        let orbit_b = b.to_elements(MASS, 0.0);

        println!();

        println!("orbit_a = {orbit_a:#?}");
        println!("orbit_b = {orbit_b:#?}");

        assert!(
            orbit_a.angle_abs_diff(&orbit_b) < 1.0,
            "Expected the diff between orbits {} to be less than 1.0",
            orbit_a.angle_abs_diff(&orbit_b)
        );
    }

    #[test]
    fn insanity_check() {
        let r = vec3(-661207000.0, 348865760.0, 13339082.0);
        let v1 = vec3(-6.25e-7, -1.1828758e-6, 1.9684657e-8);
        let v2 = vec3(-6.1500003e-7, -1.1828758e-6, 1.9684657e-8);

        println!("r = {r:?}");
        println!("v1 = {v1:?}");
        println!("v2 = {v2:?}");

        println!("r.dot(v1) = {}", r.dot(v1));
        println!("r.dot(v2) = {}", r.dot(v2));
    }
}
