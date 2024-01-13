use crate::constants::{G, TWO_PI};
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

pub fn period(a: Num, mass: Num) -> Num {
    TWO_PI * (a.powi(3) / standard_gravitational_parameter(mass)).sqrt()
}
