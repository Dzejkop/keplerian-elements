use crate::Vec3;

pub const fn zup2yup(Vec3 { x, y, z }: Vec3) -> Vec3 {
    Vec3 { x, y: z, z: y }
}

pub const fn yup2zup(Vec3 { x, y, z }: Vec3) -> Vec3 {
    Vec3 { x, y: z, z: y }
}
