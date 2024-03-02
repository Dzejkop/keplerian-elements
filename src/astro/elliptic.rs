use super::standard_gravitational_parameter;
use crate::math::newton_approx;
use crate::Num;

/// Eccentric Anomaly (E) is given by the equation:
/// M = E - e * sin(E)
/// where
/// M is the mean anomaly
/// e is the eccentricity
///
/// https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/elliptical-orbits.html#equation-eq-keplers-equation-ellipse
pub fn estimate_anomaly(
    // Mean anomaly
    M: Num,
    // Eccentricity
    e: Num,
    tolerance: Num,
) -> Num {
    newton_approx(
        // f(E) = E - e*sin(E) - M
        |E| E - (e * E.sin()) - M,
        // f'(E) = 1 - e*cos(E)
        |E| 1.0 - (e * E.cos()),
        M,
        tolerance,
    )
}

/// Mean motion
/// https://en.wikipedia.org/wiki/Mean_anomaly
pub fn mean_motion(h: Num, e: Num, mass: Num) -> Num {
    let μ = standard_gravitational_parameter(mass);

    (μ.powi(2) / h.powi(3)) * (1.0 - e.powi(2)).powi(3).sqrt()
}

pub fn true_anomaly(E: Num, e: Num) -> Num {
    // Circular (practically unattainable), elliptic or parabolic (practically unattainable)
    // https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/elliptical-orbits.html#equation-eq-eccentric-anomaly-true-anomaly-ellipse
    2.0 * ((E / 2.0).tan() / ((1.0 - e) / (1.0 + e)).sqrt()).atan()
}
