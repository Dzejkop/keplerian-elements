use bevy::prelude::*;
use keplerian_elements::utils::zup2yup;
use smooth_bevy_cameras::LookTransform;

use super::{mass2radius, FocusMode, Planet, Star, State};

pub fn epoch(time: Res<Time>, mut state: ResMut<State>) {
    if state.update_epoch {
        state.epoch += state.epoch_scale * time.delta_seconds();
    }
}

pub fn planets(
    mut query: Query<(&mut Transform, &mut Planet)>,
    state: Res<State>,
) {
    for (mut transform, mut planet) in query.iter_mut() {
        let dt = state.epoch - planet.last_update_epoch;

        planet.state_vectors = planet.state_vectors.propagate_kepler(
            dt,
            state.star_mass,
            state.tolerance,
        );
        planet.last_update_epoch = state.epoch;

        let position = zup2yup(planet.state_vectors.position);

        transform.translation = position * state.distance_scaling;
        transform.scale = Vec3::ONE * mass2radius(state.as_ref(), planet.mass);
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
