use crate::Vec3;

pub fn zup2yup(Vec3 { x, y, z }: Vec3) -> Vec3 {
    Vec3 { x: x, y: z, z: y }
}

pub fn yup2zup(Vec3 { x, y, z }: Vec3) -> Vec3 {
    Vec3 { x: x, y: z, z: y }
}
