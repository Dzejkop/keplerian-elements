use crate::constants::G;
use crate::Num;

pub mod elliptic;
pub mod hyperbolic;

/// https://en.wikipedia.org/wiki/Standard_gravitational_parameter
#[inline]
pub fn standard_gravitational_parameter(mass: Num) -> Num {
    G * mass
}

pub fn soi(r: Num, m1: Num, m2: Num) -> Num {
    r * (m1 / m2).powf(2.0 / 5.0)
}
