use bevy::gizmos::gizmos::Gizmos;
use bevy::render::color::Color;
use glam::{Quat, Vec3};

const ARROW_WING_LENGTH: f32 = 1.0;
const ARROW_WING_ANGLE: f32 = 30.0;

pub struct DebugArrows<'a, 'g> {
    lines: &'a mut Gizmos<'g>,
    camera_position: Vec3,
}

impl<'a, 'g> DebugArrows<'a, 'g> {
    pub fn new(lines: &'a mut Gizmos<'g>, camera_position: Vec3) -> Self {
        Self {
            lines,
            camera_position,
        }
    }

    pub fn draw_arrow(&mut self, start: Vec3, end: Vec3, color: Color) {
        self.lines.line(start, end, color);

        let to_start = (start - end).normalize();
        let axis_start = closest_point(self.camera_position, start, end);
        let rot_axis = (self.camera_position - axis_start).normalize();

        let angle = deg2rad(ARROW_WING_ANGLE);
        let rot_1 = Quat::from_axis_angle(rot_axis, angle);
        let rot_2 = Quat::from_axis_angle(rot_axis, -angle);

        let wing_1 = (rot_1 * to_start) * ARROW_WING_LENGTH + end;
        let wing_2 = (rot_2 * to_start) * ARROW_WING_LENGTH + end;

        self.lines.line(end, wing_1, color);
        self.lines.line(end, wing_2, color);
    }
}

/// Finds the closest point on the line segment defined by `a` and `b` to `pos`.
/// By definition the lines given by a and b and the pos and found point must be perpendicular.
fn closest_point(pos: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let ab = b - a;
    let ap = pos - a;

    let t = ap.dot(ab) / ab.dot(ab);

    if t < 0.0 {
        a
    } else if t > 1.0 {
        b
    } else {
        a + ab * t
    }
}

fn deg2rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}
