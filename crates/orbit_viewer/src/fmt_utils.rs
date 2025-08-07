use std::fmt;

use glam::Vec3;

pub struct VecDisplay(pub Vec3);

impl fmt::Display for VecDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}]", self.0.x, self.0.y, self.0.z)
    }
}

impl fmt::Debug for VecDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VecDisplay").field(&self.0).finish()
    }
}
