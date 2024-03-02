use std::f32::consts::PI;

use bevy::prelude::*;
use keplerian_elements::utils::zup2yup;
use keplerian_elements::StateVectors;

use super::State;
use crate::debug_arrows::DebugArrows;
use crate::Planet;

pub fn orbits(
    mut lines: Gizmos,
    planets: Query<(&Planet, &Handle<StandardMaterial>)>,
    materials: Res<Assets<StandardMaterial>>,
    state: Res<State>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    if !state.draw_orbits {
        return;
    }

    let camera = camera.single();
    let camera_position = camera.translation();

    for (planet, mat) in planets.iter() {
        let color = materials.get(mat).unwrap().base_color;

        let first_position =
            zup2yup(planet.state_vectors.position) * state.distance_scaling;
        let mut prev_position = first_position.clone();

        let period = planet.state_vectors.period(state.star_mass);
        let step = period / state.orbit_subdivisions as f32;

        let orbit_positions = (0..state.orbit_subdivisions)
            .map(|i| {
                let dt = i as f32 * step;

                let StateVectors { position, .. } = planet
                    .state_vectors
                    .propagate_kepler(dt, state.star_mass, state.tolerance);

                position
            })
            .collect::<Vec<_>>();

        for pos in orbit_positions {
            let position = zup2yup(pos) * state.distance_scaling;

            lines.line(prev_position, position, color);

            prev_position = position;
        }

        // Close the loop
        lines.line(prev_position, first_position, color);

        let mut debug_arrows = DebugArrows::new(&mut lines, camera_position);

        if state.show_position_and_velocity {
            let StateVectors { position, velocity } = planet.state_vectors;

            let position = zup2yup(position);
            let velocity = zup2yup(velocity);

            let p = position * state.distance_scaling;
            let v = velocity * state.distance_scaling * state.velocity_scaling;

            debug_arrows.draw_arrow(Vec3::ZERO, p, color);
            debug_arrows.draw_arrow(p, p + v, Color::RED);
        }

        let orbit = planet.orbit;
        if state.show_nodes {
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.ascending_node(state.star_mass))
                    * state.distance_scaling,
                Color::YELLOW_GREEN,
            );
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.descending_node(state.star_mass))
                    * state.distance_scaling,
                Color::YELLOW,
            );
        }

        if state.show_peri_and_apo_apsis {
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.periapsis(state.star_mass))
                    * state.distance_scaling,
                Color::WHITE,
            );
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.apoapsis(state.star_mass))
                    * state.distance_scaling,
                Color::WHITE,
            );
        }
    }
}

pub fn soi(
    mut lines: Gizmos,
    planets: Query<&Planet>,
    state: Res<State>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    if !state.draw_soi {
        return;
    }

    let camera = camera.single();

    let camera_position = camera.translation();

    for planet in planets.iter() {
        let r = planet.state_vectors.position.length();

        let soi =
            keplerian_elements::astro::soi(r, planet.mass, state.star_mass)
                * state.distance_scaling;

        let pos =
            zup2yup(planet.state_vectors.position) * state.distance_scaling;

        let to_camera = (camera_position - pos).normalize();
        let planet_camera_radial = to_camera.cross(pos).normalize();

        let mut prev_pos = pos + planet_camera_radial * soi;
        for i in 0..=100 {
            let t = i as f32 * 2.0 * PI / 100.0;

            let rot_matrix = Mat3::from_axis_angle(to_camera, t);

            let p = rot_matrix * planet_camera_radial;
            let p = pos + p * soi;

            lines.line(prev_pos, p, Color::WHITE);

            prev_pos = p;
        }
    }
}

pub fn axis(mut lines: Gizmos, state: Res<State>) {
    if !state.draw_axis {
        return;
    }

    const ORIGIN: Vec3 = Vec3::ZERO;

    lines.line(ORIGIN, ORIGIN + state.axis_scale * Vec3::X, Color::RED);
    lines.line(ORIGIN, ORIGIN + state.axis_scale * Vec3::Y, Color::GREEN);
    lines.line(ORIGIN, ORIGIN + state.axis_scale * Vec3::Z, Color::BLUE);
}
