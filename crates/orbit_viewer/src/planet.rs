use bevy::prelude::*;
use keplerian_elements::{KeplerianElements, StateVectors};

#[derive(Default, Component)]
pub struct CelestialBody {
    pub state_vectors: StateVectors,
    pub last_update_epoch: f32,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct CelestialMass(pub f32);

#[derive(Debug, Clone, Copy, Component)]
pub struct CelestialRadius(pub f32);

#[derive(Debug, Clone, Copy, Component)]
pub struct CelestialParent(pub Entity);

impl CelestialBody {
    pub fn from_elements(
        orbit: KeplerianElements,
        central_mass: f32,
        tolerance: f32,
    ) -> Self {
        let state_vectors =
            orbit.state_vectors_at_epoch(central_mass, 0.0, tolerance);

        CelestialBody {
            state_vectors,
            last_update_epoch: 0.0,
        }
    }
}
