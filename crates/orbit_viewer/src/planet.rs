use bevy::prelude::*;
use keplerian_elements::{KeplerianElements, StateVectors};

#[derive(Bundle)]
pub struct PlanetBundle {
    pub planet: Planet,
    pub planet_mass: PlanetMass,
    pub planet_parent: PlanetParent,
}

#[derive(Component)]
pub struct Planet {
    pub orbit: KeplerianElements,
    pub state_vectors: StateVectors,
    pub last_update_epoch: f32,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct PlanetMass(pub f32);

#[derive(Debug, Clone, Copy, Component)]
pub struct PlanetParent(pub Option<Entity>);

impl PlanetBundle {
    pub fn new_from_orbit(
        orbit: KeplerianElements,
        mass: f32,
        central_mass: f32,
        tolerance: f32,
    ) -> Self {
        PlanetBuilder::new(orbit, mass).build(central_mass, tolerance)
    }

    pub fn builder(orbit: KeplerianElements, mass: f32) -> PlanetBuilder {
        PlanetBuilder::new(orbit, mass)
    }
}

pub struct PlanetBuilder {
    planet: Planet,
    planet_mass: PlanetMass,
    planet_parent: PlanetParent,
}

impl PlanetBuilder {
    pub fn new(orbit: KeplerianElements, mass: f32) -> Self {
        Self {
            planet: Planet {
                orbit,
                state_vectors: StateVectors::default(),
                last_update_epoch: 0.0,
            },
            planet_mass: PlanetMass(mass),
            planet_parent: PlanetParent(None),
        }
    }

    pub fn with_parent(mut self, parent: Entity) -> Self {
        self.planet_parent.0 = Some(parent);
        self
    }

    pub fn build(mut self, central_mass: f32, tolerance: f32) -> PlanetBundle {
        self.planet.state_vectors = self.planet.orbit.state_vectors_at_epoch(
            central_mass,
            0.0,
            tolerance,
        );

        let PlanetBuilder {
            planet,
            planet_mass,
            planet_parent,
        } = self;

        PlanetBundle {
            planet,
            planet_mass,
            planet_parent,
        }
    }
}
