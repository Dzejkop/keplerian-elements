use std::f32::consts::PI;

use bevy::prelude::*;
use keplerian_elements::utils::zup2yup;
use keplerian_elements::StateVectors;

use crate::debug_arrows::DebugArrows;
use crate::planet::{CelestialMass, CelestialParent};
use crate::trajectory::{
    SimulatorSettings, SimulatorState, TrajectorySimulator,
};
use crate::{CelestialBody, State};

const SOI_SEGMENTS: usize = 50;

pub fn orbits(
    mut lines: Gizmos,
    planets: Query<Entity, With<CelestialBody>>,
    planet_data: Query<(
        &CelestialBody,
        &CelestialParent,
        &Handle<StandardMaterial>,
    )>,
    planet_masses: Query<&CelestialMass>,
    transforms: Query<&Transform>,
    materials: Res<Assets<StandardMaterial>>,
    state: Res<State>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    if !state.draw_orbits {
        return;
    }

    let camera = camera.single();
    let camera_position = camera.translation();

    for planet_entity in planets.iter() {
        let Ok((planet, parent, mat)) = planet_data.get(planet_entity) else {
            continue;
        };

        let central_mass =
            planet_masses.get(parent.0).expect("Missing parent").0;

        let color = materials.get(mat).unwrap().base_color;

        let offset = {
            let transform = transforms
                .get(parent.0)
                .expect("Parent planet does not exist");

            transform.translation
        };

        let first_position = offset
            + zup2yup(planet.state_vectors.position) * state.distance_scaling;
        let mut prev_position = first_position.clone();

        let period = planet.state_vectors.period(central_mass);
        let step = period / state.orbit_subdivisions as f32;

        let orbit_positions = (0..state.orbit_subdivisions)
            .map(|i| {
                let dt = i as f32 * step;

                let StateVectors { position, .. } = planet
                    .state_vectors
                    .propagate_kepler(dt, central_mass, state.tolerance);

                position
            })
            .collect::<Vec<_>>();

        for pos in orbit_positions {
            let position = offset + zup2yup(pos) * state.distance_scaling;

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
    }
}

pub fn soi(
    mut lines: Gizmos,
    planets: Query<(Entity, &CelestialBody, &CelestialParent)>,
    transforms: Query<&Transform>,
    planet_masses: Query<&CelestialMass>,
    state: Res<State>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    if !state.draw_soi {
        return;
    }

    let camera = camera.single();

    let camera_position = camera.translation();

    for (entity, planet, parent) in planets.iter() {
        let r = planet.state_vectors.position.length();

        let central_mass = planet_masses.get(parent.0).unwrap().0;

        let mass = planet_masses.get(entity).unwrap();

        let offset = {
            let transform = transforms
                .get(parent.0)
                .expect("Parent planet does not exist");

            transform.translation
        };

        let soi = keplerian_elements::astro::soi(r, mass.0, central_mass)
            * state.distance_scaling;

        let pos = offset
            + zup2yup(planet.state_vectors.position) * state.distance_scaling;

        let to_camera = (camera_position - pos).normalize();
        let planet_camera_radial = to_camera.cross(pos).normalize();

        let mut prev_pos = pos + planet_camera_radial * soi;
        for i in 0..=SOI_SEGMENTS {
            let t = i as f32 * 2.0 * PI / SOI_SEGMENTS as f32;

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

pub fn trajectory(
    planets: Query<&CelestialBody>,
    masses: Query<&CelestialMass>,
    state: Res<State>,
    mut gizmos: Gizmos,
    simulator_state: Res<SimulatorState>,
    simulator: Res<TrajectorySimulator>,
    simulator_settings: Res<SimulatorSettings>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    if !simulator_state.enabled {
        return;
    }

    // TODO: Move to settings
    const SIMULATOR_LOC_SCALE: f32 = 500.0;

    let origin = zup2yup(simulator.origin * state.distance_scaling);

    gizmos.line(
        origin,
        origin + Vec3::Y * SIMULATOR_LOC_SCALE,
        Color::YELLOW,
    );
    gizmos.line(
        origin,
        origin + Vec3::X * SIMULATOR_LOC_SCALE,
        Color::YELLOW,
    );
    gizmos.line(
        origin,
        origin + Vec3::Z * SIMULATOR_LOC_SCALE,
        Color::YELLOW,
    );

    let camera = camera.single();

    let camera_position = camera.translation();

    let mut debug_arrows = DebugArrows::new(&mut gizmos, camera_position);

    let v = zup2yup(simulator.velocity) * state.velocity_scaling;

    // Draw velocity
    debug_arrows.draw_arrow(origin, origin + v, Color::YELLOW_GREEN);

    let segments = &simulator.segments;
    for segment in segments {
        let mut pos =
            zup2yup(segment.entry_sv.position) * state.distance_scaling;

        for i in 0..=simulator_settings.max_steps {
            let t = i as f32 * simulator_settings.epoch_state;

            let central_mass = masses.get(segment.parent).unwrap().0;

            let offset = {
                let planet = planets.get(segment.parent).unwrap();
                planet.state_vectors.position
            };

            let mut entry_sv = segment.entry_sv.clone();
            entry_sv.position -= offset; // Move to the orbital frame

            let sv =
                entry_sv.try_propagate_kepler(t, central_mass, state.tolerance);

            let sv = match sv {
                Some(sv) => sv,
                None => {
                    error!("Failed to propagate kepler");
                    break;
                }
            };

            let next_pos =
                zup2yup(sv.position + offset) * state.distance_scaling;

            gizmos.line(pos, next_pos, Color::WHITE);

            pos = next_pos;
        }
    }
}
