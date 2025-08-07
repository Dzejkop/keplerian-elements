use bevy::prelude::*;
use keplerian_elements::{astro, StateVectors};

use crate::planet::{CelestialBody, CelestialMass, CelestialParent};
use crate::{Epoch, State};

const PERIOD_SAMPLES: usize = 500;

#[derive(Debug, Clone, Copy, Default, Event)]
pub struct RecalculateTrajectory;

#[derive(Resource, Default)]
pub struct SimulatorState {
    pub enabled: bool,
}

#[derive(Resource)]
pub struct SimulatorSettings {
    pub epoch_state: f32,
    pub max_steps: usize,
    pub auto_recalculate: bool,
}

impl Default for SimulatorSettings {
    fn default() -> Self {
        Self {
            epoch_state: 1000.00,
            max_steps: 1000,
            auto_recalculate: false,
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
    pub parent: Entity,
}

pub fn recalculate(
    state: Res<State>,
    epoch: Res<Epoch>,
    planets: Query<(
        Entity,
        &CelestialBody,
        &CelestialMass,
        Option<&CelestialParent>,
    )>,
    masses: Query<&CelestialMass>,
    mut trajectory_simulator: ResMut<TrajectorySimulator>,
    settings: Res<SimulatorSettings>,
    mut recalculate_event_reader: EventReader<RecalculateTrajectory>,
) {
    if !settings.auto_recalculate
        && recalculate_event_reader.read().count() == 0
    {
        return;
    }

    let starting_sv = StateVectors::new(
        trajectory_simulator.origin,
        trajectory_simulator.velocity,
    );

    let parent = find_soi_at_position(&starting_sv, &planets);

    trajectory_simulator.segments.clear();

    let mut segment = TrajectorySegment {
        entry: epoch.0,
        entry_sv: starting_sv.clone(),
        parent,
    };

    loop {
        let central_mass = masses
            .get(segment.parent)
            .expect("Missing mass on parent")
            .0;

        let eccentricity = segment.entry_sv.eccentricity(central_mass);

        if eccentricity < 1.0 {
            let period = segment.entry_sv.period(central_mass);

            let step = period / PERIOD_SAMPLES as f32;

            for i in 0..PERIOD_SAMPLES {
                let dt = i as f32 * step;

                let new_sv = segment.entry_sv.propagate_kepler(dt, central_mass, state.tolerance);

                let new_parent = find_soi_at_position(&new_sv, &planets);

                if new_parent != segment.parent {
                    segment.entry_sv = new_sv;
                    segment.parent = new_parent;
                    segment.entry += dt;

                    break;
                }
            }

            // Circular or elliptic
        } else {
            // Parabolic or hyperbolic
        }

        trajectory_simulator.segments.push(segment);

        break;
    }
}

fn find_soi_at_position(
    starting_sv: &StateVectors,
    planets: &Query<(
        Entity,
        &CelestialBody,
        &CelestialMass,
        Option<&CelestialParent>,
    )>,
) -> Entity {
    let mut soi_parent = None;
    let mut d = f32::MAX;

    for (entity, planet, mass, parent) in planets.iter() {
        let Some(parent) = parent else {
            // No parent indicates the central star
            // in this case we set the entity
            // but keep the distance as f32::MAX
            soi_parent = Some(entity);

            continue;
        };

        let central_mass =
            planets.get(parent.0).expect("No mass for parent").2 .0;

        let offset = {
            let (_, parent_planet, _, _) =
                planets.get(parent.0).expect("Parent planet does not exist");

            parent_planet.state_vectors.position
        };

        let real_soi_center = planet.state_vectors.position + offset;
        let soi = astro::soi(
            planet.state_vectors.position.length(),
            mass.0,
            central_mass,
        );

        let new_d = (real_soi_center - starting_sv.position).length();
        if new_d < soi && new_d < d {
            soi_parent = Some(entity);
            d = new_d;
        }
    }

    soi_parent.expect("Found no parent entity!")
}
