use bevy::prelude::*;
use keplerian_elements::{astro, StateVectors};

use crate::planet::{Planet, PlanetMass, PlanetParent};
use crate::{Epoch, State};

#[derive(Debug, Clone, Copy, Default, Event)]
pub struct RecalculateTrajectory;

#[derive(Resource, Default)]
pub struct SimulatorState {
    pub enabled: bool,
}

#[derive(Resource)]
pub struct SimulatorSettings {
    pub step: f32,
    pub max_steps: usize,
}

impl Default for SimulatorSettings {
    fn default() -> Self {
        Self {
            step: 60000002048.00,
            max_steps: 1000,
        }
    }
}

#[derive(Resource, Default)]
pub struct TrajectorySimulator {
    pub origin: Vec3,
    pub velocity: Vec3,

    pub segments: Vec<TrajectorySegment>,
}

pub struct TrajectorySegment {
    // Epoch of the segment entrypoint
    pub entry: f32,
    // State vectors at the entrypoint
    pub entry_sv: StateVectors,
    // Parent of the given segment
    pub parent: Option<Entity>,
}

pub fn recalculate(
    state: Res<State>,
    epoch: Res<Epoch>,
    planets: Query<(Entity, &Planet, &PlanetMass, &PlanetParent)>,
    mut trajectory_simulator: ResMut<TrajectorySimulator>,
    _settings: Res<SimulatorSettings>,
    mut recalculate_event_reader: EventReader<RecalculateTrajectory>,
) {
    if recalculate_event_reader.read().count() == 0 {
        return;
    }

    info!("Recalculating trajectory...");
    let starting_sv = StateVectors::new(
        trajectory_simulator.origin,
        trajectory_simulator.velocity,
    );

    let parent = find_parent(&starting_sv, &state, &planets);

    trajectory_simulator.segments.clear();

    trajectory_simulator.segments.push(TrajectorySegment {
        entry: epoch.0,
        entry_sv: starting_sv.clone(),
        parent,
    });

    // Algorithm:
    // 1. Propagate the segment until: a) it loops around, b) it leaves the SOI, c) it intersects an SOI of a different planet
    // 2. If:
    //      a) is true, stop the propagation and add the segment to the list
    //      b) is true, find SOI exit time and add a second segment with parent of the parent
    //      c)
}

fn find_parent(
    starting_sv: &StateVectors,
    state: &Res<State>,
    planets: &Query<(Entity, &Planet, &PlanetMass, &PlanetParent)>,
) -> Option<Entity> {
    for (entity, planet, mass, parent) in planets.iter() {
        let central_mass = if let Some(parent) = parent.0 {
            planets.get(parent).expect("No mass for parent").2 .0
        } else {
            state.star_mass
        };

        let offset = if let Some(parent) = parent.0 {
            let (_, parent_planet, _, _) =
                planets.get(parent).expect("Parent planet does not exist");

            parent_planet.state_vectors.position
        } else {
            Vec3::ZERO
        };

        let real_soi_center = planet.state_vectors.position + offset;
        let soi = astro::soi(
            planet.state_vectors.position.length(),
            mass.0,
            central_mass,
        );

        let d = (real_soi_center - starting_sv.position).length();
        if d < soi {
            return Some(entity);
        }
    }

    None
}
