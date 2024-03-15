use bevy::core_pipeline::bloom::BloomSettings;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use keplerian_elements::KeplerianElements;
use planet::{Planet, PlanetMass, PlanetParent};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;
use trajectory::RecalculateTrajectory;

const BASE_TOLERANCE: f32 = 0.01;

mod debug_arrows;
mod draw;
mod planet;
mod trajectory;
mod ui;
mod update;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::new(false))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .init_resource::<ui::UiState>()
        .add_systems(Update, ui::render)
        .add_systems(Update, ui::simulator_window)
        .add_systems(Update, ui::simulator_settings_window)
        .add_systems(Update, update::epoch)
        .add_systems(Update, update::planets)
        .add_systems(Update, update::planet_scale)
        .add_systems(Update, update::camera_focus)
        .add_systems(Update, draw::orbits)
        .add_systems(Update, draw::axis)
        .add_systems(Update, draw::soi)
        .add_systems(Update, draw::trajectory)
        .init_resource::<trajectory::TrajectorySimulator>()
        .init_resource::<trajectory::SimulatorSettings>()
        .init_resource::<trajectory::SimulatorState>()
        .add_event::<RecalculateTrajectory>()
        .add_systems(Update, trajectory::recalculate)
        .run();
}

#[derive(Resource)]
struct State {
    tolerance: f32,

    update_epoch: bool,
    update_planets: bool,
    epoch_scale: f32,

    draw_orbits: bool,
    orbit_subdivisions: u32,
    show_nodes: bool,
    show_peri_and_apo_apsis: bool,
    show_position_and_velocity: bool,

    draw_soi: bool,

    draw_axis: bool,
    axis_scale: f32,

    scale_scaling: f32,
    distance_scaling: f32,
    velocity_scaling: f32,
    focus_mode: FocusMode,
}

#[derive(Debug, Clone, Copy, Resource)]
pub struct Epoch(pub f32);

#[derive(Debug, Clone, PartialEq, Eq)]
enum FocusMode {
    Sun,
    // By name - inefficient, but I don't care
    Planet(String),
}

#[derive(Component)]
struct Star;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::BLACK));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.01,
    });

    commands.insert_resource(State {
        tolerance: BASE_TOLERANCE,
        epoch_scale: 1.0,
        update_epoch: true,
        update_planets: true,
        draw_orbits: true,
        orbit_subdivisions: 100,
        show_nodes: false,
        show_peri_and_apo_apsis: false,
        show_position_and_velocity: false,
        draw_soi: true,
        draw_axis: true,
        axis_scale: 10000.0,
        scale_scaling: 1e-28,
        distance_scaling: 1e-6,
        velocity_scaling: 1e11,
        focus_mode: FocusMode::Sun,
    });

    commands.insert_resource(Epoch(0.0));

    let sphere = meshes.add(
        shape::Icosphere {
            radius: 1.0,
            subdivisions: 4,
        }
        .try_into()
        .unwrap(),
    );

    spawn_kerbol_system(&mut commands, sphere, materials.as_mut());

    commands
        .spawn(Camera3dBundle::default())
        .insert(BloomSettings::OLD_SCHOOL)
        .insert(OrbitCameraBundle::new(
            {
                let mut controller = OrbitCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 1.0;
                controller.mouse_translate_sensitivity = Vec2::ONE * 10.0;

                controller
            },
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ));
}

fn spawn_kerbol_system(
    commands: &mut Commands,
    sphere: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut star_material = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            emissive: color * 100.0,
            unlit: true,
            ..Default::default()
        })
    };

    let kerbol_mass = 1.756546e28;
    let kerbol = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: star_material(Color::YELLOW),
            ..Default::default()
        })
        .insert(Planet::default())
        .insert(PlanetMass(kerbol_mass))
        .insert(NotShadowCaster)
        .insert(Star)
        .insert(Name::new("Kerbol"))
        .id();

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 100000.0,
            range: 100000.0,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    let mut planet_material = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            emissive: color,
            perceptual_roughness: 1.0,
            ..Default::default()
        })
    };

    let kerbin_mass = 5.2915158e22;
    let kerbin = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::RED),
            ..Default::default()
        })
        .insert(Planet::from_elements(
            KeplerianElements {
                semi_major_axis: 13_599_840_256.0,
                eccentricity: 0.0,
                inclination: 0.0,
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
                mean_anomaly_at_epoch: 3.14,
                epoch: 0.0, // Example epoch year
            },
            kerbol_mass,
            BASE_TOLERANCE,
        ))
        .insert(PlanetMass(kerbin_mass))
        .insert(PlanetParent(kerbol))
        .insert(Name::new("Kerbin"))
        .id();

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BLUE),
            ..Default::default()
        })
        .insert(Planet::from_elements(
            KeplerianElements {
                semi_major_axis: 12_000_000.0,
                eccentricity: 0.0,
                inclination: 0.0,
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
                mean_anomaly_at_epoch: 1.7,
                epoch: 0.0, // Example epoch year
            },
            kerbin_mass,
            BASE_TOLERANCE,
        ))
        .insert(PlanetParent(kerbin))
        .insert(PlanetMass(9.7599066e20))
        .insert(Name::new("Mun"));
}
