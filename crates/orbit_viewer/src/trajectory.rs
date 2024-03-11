use bevy::prelude::*;
use keplerian_elements::StateVectors;

use crate::Epoch;

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
    epoch: Res<Epoch>,
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

    trajectory_simulator.segments.clear();

    trajectory_simulator.segments.push(TrajectorySegment {
        entry: epoch.0,
        entry_sv: starting_sv.clone(),
        parent: None,
    });
}
