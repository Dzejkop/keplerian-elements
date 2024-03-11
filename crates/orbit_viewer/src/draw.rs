use std::f32::consts::PI;

use bevy::prelude::*;
use keplerian_elements::utils::zup2yup;
use keplerian_elements::StateVectors;

use crate::debug_arrows::DebugArrows;
use crate::planet::{PlanetMass, PlanetParent};
use crate::trajectory::{
    SimulatorSettings, SimulatorState, TrajectorySimulator,
};
use crate::{Planet, State};

const SOI_SEGMENTS: usize = 50;

pub fn orbits(
    mut lines: Gizmos,
    planets: Query<Entity, With<Planet>>,
    planet_data: Query<(&Planet, &PlanetParent, &Handle<StandardMaterial>)>,
    planet_masses: Query<&PlanetMass>,
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
            warn!("Planet missing planet data");
            continue;
        };

        let central_mass = if let Some(parent) = parent.0 {
            planet_masses.get(parent).expect("No mass for parent").0
        } else {
            state.star_mass
        };

        let color = materials.get(mat).unwrap().base_color;

        let offset = if let Some(parent) = parent.0 {
            let transform = transforms
                .get(parent)
                .expect("Parent planet does not exist");

            transform.translation
        } else {
            Vec3::ZERO
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

        let orbit = planet.orbit;
        if state.show_nodes {
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.ascending_node(central_mass))
                    * state.distance_scaling,
                Color::YELLOW_GREEN,
            );
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.descending_node(central_mass))
                    * state.distance_scaling,
                Color::YELLOW,
            );
        }

        if state.show_peri_and_apo_apsis {
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.periapsis(central_mass)) * state.distance_scaling,
                Color::WHITE,
            );
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.apoapsis(central_mass)) * state.distance_scaling,
                Color::WHITE,
            );
        }
    }
}

pub fn soi(
    mut lines: Gizmos,
    planets: Query<(Entity, &Planet, &PlanetParent)>,
    transforms: Query<&Transform>,
    planet_masses: Query<&PlanetMass>,
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

        let central_mass = if let Some(parent) = parent.0 {
            planet_masses.get(parent).unwrap().0
        } else {
            state.star_mass
        };
        let mass = planet_masses.get(entity).unwrap();

        let offset = if let Some(parent) = parent.0 {
            let transform = transforms
                .get(parent)
                .expect("Parent planet does not exist");

            transform.translation
        } else {
            Vec3::ZERO
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
            let t = i as f32 * simulator_settings.step;

            let sv = segment.entry_sv.try_propagate_kepler(
                t,
                state.star_mass,
                state.tolerance,
            );

            let sv = match sv {
                Some(sv) => sv,
                None => {
                    error!("Failed to propagate kepler");
                    break;
                }
            };

            let next_pos = zup2yup(sv.position) * state.distance_scaling;

            gizmos.line(pos, next_pos, Color::WHITE);

            pos = next_pos;
        }
    }
}
