use crate::Num;

/// Gravitational constant
pub const G: Num = 6.67430e-11;

/// Astronomical unit in km
pub const AU: Num = 1.496e+8;

#[cfg(feature = "f32")]
pub use std::f32::consts::PI;
#[cfg(feature = "f64")]
pub use std::f64::consts::PI;

pub const TWO_PI: Num = 2.0 * PI;
