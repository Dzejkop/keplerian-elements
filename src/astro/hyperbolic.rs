use super::standard_gravitational_parameter;
use crate::math::newton_approx;
use crate::Num;

/// Hyperbolic Anomaly (F) is given by the equation:
/// M = e * sinh(F) - F
/// where
/// M is the hyperbolic mean anomaly
/// e is the eccentricity
///
/// https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-keplers-equation
pub fn estimate_anomaly(M: Num, e: Num, tolerance: Num) -> Num {
    newton_approx(
        // f(F) = e * sinh(F) - F - M
        |F| (e * F.sinh()) - F - M,
        // f'(F) = e * cosh(F) - 1
        |F| e * F.cosh() - 1.0,
        M,
        tolerance,
    )
}

/// Hyperbolic mean motion
/// SRC: https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-hyperbolic-mean-anomaly
pub fn mean_motion(h: Num, e: Num, mass: Num) -> Num {
    let μ = standard_gravitational_parameter(mass);

    (μ.powi(2) / h.powi(3)) * (e.powi(2) - 1.0).powi(3).sqrt()
}

pub fn true_anomaly(F: Num, e: Num) -> Num {
    // https://orbital-mechanics.space/time-since-periapsis-and-keplers-equation/hyperbolic-trajectories.html#equation-eq-eccentric-anomaly-true-anomaly-hyperbola
    2.0 * ((F / 2.0).tanh() / ((e - 1.0) / (e + 1.0)).sqrt()).atan()
}
