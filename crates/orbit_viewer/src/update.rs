use bevy::prelude::*;
use keplerian_elements::utils::zup2yup;
use smooth_bevy_cameras::LookTransform;

use super::{FocusMode, Planet, State};
use crate::planet::{PlanetMass, PlanetParent};
use crate::Epoch;

pub fn epoch(time: Res<Time>, mut epoch: ResMut<Epoch>, state: Res<State>) {
    if state.update_epoch {
        epoch.0 += state.epoch_scale * time.delta_seconds();
    }
}

pub fn planets(
    planet_entities: Query<Entity, With<Planet>>,
    mut planets: Query<&mut Planet>,
    mut transforms: Query<&mut Transform>,
    parents: Query<&PlanetParent>,
    planet_masses: Query<&PlanetMass>,
    state: Res<State>,
    epoch: Res<Epoch>,
) {
    if !state.update_epoch {
        return;
    }

    for entity in planet_entities.iter() {
        let Ok(parent) = parents.get(entity) else {
            continue;
        };

        let Ok(mut planet) = planets.get_mut(entity) else {
            continue;
        };

        let dt = epoch.0 - planet.last_update_epoch;

        let central_mass = planet_masses.get(parent.0).unwrap().0;

        let offset = {
            let transform = transforms
                .get(parent.0)
                .expect("Parent planet does not exist");

            transform.translation
        };

        let Ok(mut transform) = transforms.get_mut(entity) else {
            warn!("Planet has no transform component");
            continue;
        };

        planet.state_vectors = planet.state_vectors.propagate_kepler(
            dt,
            central_mass,
            state.tolerance,
        );
        planet.last_update_epoch = epoch.0;

        let position = zup2yup(planet.state_vectors.position);

        let new_translation = offset + position * state.distance_scaling;

        transform.translation = new_translation;
    }
}

pub fn planet_scale(
    state: Res<State>,
    mut items: Query<(&mut Transform, &PlanetMass)>,
) {
    for (mut transform, mass) in items.iter_mut() {
        transform.scale = Vec3::ONE * mass.0 * state.scale_scaling;
    }
}

pub fn camera_focus(
    mut look_transform: Query<&mut LookTransform>,
    state: Res<State>,
    planets: Query<(&GlobalTransform, &Name), With<Planet>>,
) {
    let mut look = look_transform.single_mut();

    match &state.focus_mode {
        FocusMode::Sun => {
            look.target = Vec3::ZERO;
        }
        FocusMode::Planet(focused_name) => {
            for (transform, name) in planets.iter() {
                if focused_name == name.as_ref() {
                    look.target = transform.translation();
                }
            }
        }
    }
}
