use glam::Vec3;

pub fn z_up_to_y_up(v: Vec3) -> Vec3 {
    Vec3::new(-v.x, v.z, v.y)
}
