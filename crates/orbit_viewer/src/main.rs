use bevy::core_pipeline::bloom::BloomSettings;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use keplerian_elements::KeplerianElements;
use planet::{CelestialBody, CelestialMass, CelestialParent, CelestialRadius};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;
use trajectory::RecalculateTrajectory;

const BASE_TOLERANCE: f32 = 0.01;

mod debug_arrows;
mod draw;
mod fmt_utils;
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
        .add_systems(Update, update::planets_sys)
        .add_systems(Update, update::planet_scale_sys)
        .add_systems(Update, update::camera_focus_sys)
        .add_systems(Update, draw::orbits)
        .add_systems(Update, draw::position_and_velocity)
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
        distance_scaling: 1e-7,
        velocity_scaling: 1e6,
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
                OrbitCameraController {
                    mouse_rotate_sensitivity: Vec2::ONE * 1.0,
                    mouse_translate_sensitivity: Vec2::ONE * 10.0,
                    ..OrbitCameraController::default()
                }
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
        .insert(CelestialBody::default())
        .insert(CelestialMass(kerbol_mass))
        .insert(CelestialRadius(261_600_000.0))
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

    let _moho = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::ORANGE),
            ..Default::default()
        })
        .insert(CelestialBody::from_elements(
            KeplerianElements {
                semi_major_axis: 5_263_138_304.0,
                eccentricity: 0.2,
                inclination: f32::to_radians(7.0),
                right_ascension_of_the_ascending_node: f32::to_radians(70.0),
                argument_of_periapsis: f32::to_radians(15.0),
                mean_anomaly_at_epoch: 3.14,
                epoch: 0.0, // Example epoch year
            },
            kerbol_mass,
            BASE_TOLERANCE,
        ))
        .insert(CelestialMass(2.5263314e21))
        .insert(CelestialParent(kerbol))
        .insert(CelestialRadius(250_000.0))
        .insert(Name::new("Moho"))
        .id();

    let _eve = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::PINK),
            ..Default::default()
        })
        .insert(CelestialBody::from_elements(
            KeplerianElements {
                semi_major_axis: 9_832_684_544.0,
                eccentricity: 0.01,
                inclination: f32::to_radians(2.1),
                right_ascension_of_the_ascending_node: f32::to_radians(15.0),
                argument_of_periapsis: 0.0,
                mean_anomaly_at_epoch: 3.14,
                epoch: 0.0,
            },
            kerbol_mass,
            BASE_TOLERANCE,
        ))
        .insert(CelestialMass(1.224398e23))
        .insert(CelestialParent(kerbol))
        .insert(CelestialRadius(700_000.0))
        .insert(Name::new("Eve"))
        .id();

    let kerbin_mass = 5.2915158e22;
    let kerbin = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::GREEN),
            ..Default::default()
        })
        .insert(CelestialBody::from_elements(
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
        .insert(CelestialMass(kerbin_mass))
        .insert(CelestialParent(kerbol))
        .insert(CelestialRadius(600_000.0))
        .insert(Name::new("Kerbin"))
        .id();

    let _mun = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::GRAY),
            ..Default::default()
        })
        .insert(CelestialBody::from_elements(
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
        .insert(CelestialParent(kerbin))
        .insert(CelestialMass(9.7599066e20))
        .insert(CelestialRadius(200_000.0))
        .insert(Name::new("Mun"));

    let _minmus = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::PURPLE),
            ..Default::default()
        })
        .insert(CelestialBody::from_elements(
            KeplerianElements {
                semi_major_axis: 47_000_000.0,
                eccentricity: 0.0,
                inclination: f32::to_radians(6.0),
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: f32::to_radians(48.0),
                mean_anomaly_at_epoch: 0.9,
                epoch: 0.0, // Example epoch year
            },
            kerbin_mass,
            BASE_TOLERANCE,
        ))
        .insert(CelestialParent(kerbin))
        .insert(CelestialMass(2.6457580e19))
        .insert(CelestialRadius(60_000.0))
        .insert(Name::new("Minmus"));

    let _duna = commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::RED),
            ..Default::default()
        })
        .insert(CelestialBody::from_elements(
            KeplerianElements {
                semi_major_axis: 20_726_155_264.0,
                eccentricity: 0.051,
                inclination: f32::to_radians(0.06),
                right_ascension_of_the_ascending_node: f32::to_radians(135.5),
                argument_of_periapsis: 0.0,
                mean_anomaly_at_epoch: 3.14,
                epoch: 0.0,
            },
            kerbol_mass,
            BASE_TOLERANCE,
        ))
        .insert(CelestialMass(4.5154e21))
        .insert(CelestialParent(kerbol))
        .insert(CelestialRadius(320_000.0))
        .insert(Name::new("Duna"))
        .id();

    // let _dres = commands
    //     .spawn(PbrBundle {
    //         mesh: sphere.clone(),
    //         material: planet_material(Color::GRAY),
    //         ..Default::default()
    //     })
    //     .insert(CelestialBody::from_elements(
    //         KeplerianElements {
    //             semi_major_axis: 40_839_348_203.0,
    //             eccentricity: 0.145,
    //             inclination: f32::to_radians(5.0),
    //             right_ascension_of_the_ascending_node: f32::to_radians(280.0),
    //             argument_of_periapsis: f32::to_radians(90.0),
    //             mean_anomaly_at_epoch: 3.14,
    //             epoch: 0.0,
    //         },
    //         kerbol_mass,
    //         BASE_TOLERANCE,
    //     ))
    //     .insert(CelestialMass(3.219e20))
    //     .insert(CelestialParent(kerbin))
    //     .insert(CelestialRadius(138_000.0))
    //     .insert(Name::new("Dres"))
    //     .id();

    // let _jool = commands
    //     .spawn(PbrBundle {
    //         mesh: sphere.clone(),
    //         material: planet_material(Color::BLUE),
    //         ..Default::default()
    //     })
    //     .insert(CelestialBody::from_elements(
    //         KeplerianElements {
    //             semi_major_axis: 68_775_000_000.0,
    //             eccentricity: 0.0,
    //             inclination: f32::to_radians(1.3),
    //             right_ascension_of_the_ascending_node: 0.0,
    //             argument_of_periapsis: 0.0,
    //             mean_anomaly_at_epoch: 0.0,
    //             epoch: 0.0,
    //         },
    //         kerbol_mass,
    //         BASE_TOLERANCE,
    //     ))
    //     .insert(CelestialMass(1.128e23))
    //     .insert(CelestialParent(kerbin))
    //     .insert(CelestialRadius(950_000.0))
    //     .insert(Name::new("Jool"))
    //     .id();
}
