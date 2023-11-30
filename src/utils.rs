use crate::{Mat3, Vec3, PI};

pub fn zup2yup(p: Vec3) -> Vec3 {
    let m = Mat3::from_rotation_x(-PI / 2.0);

    m.mul_vec3(p)
}

pub fn yup2zup(p: Vec3) -> Vec3 {
    let m = Mat3::from_rotation_x(PI / 2.0);

    m.mul_vec3(p)
}
