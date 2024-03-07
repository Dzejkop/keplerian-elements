use bevy::prelude::*;
use keplerian_elements::utils::zup2yup;
use smooth_bevy_cameras::LookTransform;

use super::{mass2radius, FocusMode, Planet, Star, State};
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
    for entity in planet_entities.iter() {
        let Ok(parent) = parents.get(entity) else {
            warn!("Planet has not parent component");
            continue;
        };

        let Ok(mut planet) = planets.get_mut(entity) else {
            error!("Planet has not planet component");
            continue;
        };

        let dt = epoch.0 - planet.last_update_epoch;
        let central_mass = if let Some(parent) = parent.0 {
            planet_masses.get(parent).unwrap().0
        } else {
            state.star_mass
        };

        let offset = if let Some(parent) = parent.0 {
            let transform = transforms
                .get(parent)
                .expect("Parent planet does not exist");

            transform.translation
        } else {
            Vec3::ZERO
        };

        let Ok(mut transform) = transforms.get_mut(entity) else {
            warn!("Planet has no transform component");
            continue;
        };

        let planet_mass = planet_masses.get(entity).unwrap().0;

        planet.state_vectors = planet.state_vectors.propagate_kepler(
            dt,
            central_mass,
            state.tolerance,
        );
        planet.last_update_epoch = epoch.0;

        let position = zup2yup(planet.state_vectors.position);

        transform.translation = offset + position * state.distance_scaling;
        transform.scale =
            Vec3::ONE * 0.1 * mass2radius(state.as_ref(), planet_mass);
    }
}

pub fn star(mut query: Query<&mut Transform, With<Star>>, state: Res<State>) {
    for mut transform in query.iter_mut() {
        transform.scale =
            Vec3::ONE * mass2radius(state.as_ref(), state.star_mass);
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
