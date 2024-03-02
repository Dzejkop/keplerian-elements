use bevy::core_pipeline::bloom::BloomSettings;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use keplerian_elements::constants::AU;
use keplerian_elements::{KeplerianElements, StateVectors};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

const USE_REAL_SOLAR_SYSTEM: bool = true;
const BASE_TOLERANCE: f32 = 0.01;
const STAR_MASS: f32 = 1.989e20;

mod debug_arrows;
mod draw;
mod ui;
mod update;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::new(false))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, ui::render)
        .add_systems(Update, update::epoch)
        .add_systems(Update, update::planets)
        .add_systems(Update, update::star)
        .add_systems(Update, update::camera_focus)
        .add_systems(Update, draw::orbits)
        .add_systems(Update, draw::axis)
        .add_systems(Update, draw::soi)
        .run();
}

#[derive(Resource)]
struct State {
    star_mass: f32,
    tolerance: f32,

    epoch: f32,
    update_epoch: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum FocusMode {
    Sun,
    // By name - inefficient, but I don't care
    Planet(String),
}

#[derive(Component)]
struct Planet {
    orbit: KeplerianElements,
    state_vectors: StateVectors,
    mass: f32,
    last_update_epoch: f32,
}

impl Planet {
    pub fn new_from_orbit(
        orbit: KeplerianElements,
        mass: f32,
        central_mass: f32,
        tolerance: f32,
    ) -> Self {
        Self {
            orbit,
            state_vectors: orbit.state_vectors_at_epoch(
                central_mass,
                0.0,
                tolerance,
            ),
            mass,
            last_update_epoch: 0.0,
        }
    }
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
        // Sun Mass
        star_mass: STAR_MASS,
        epoch: 0.0,
        epoch_scale: 1000.0,
        update_epoch: true,
        draw_orbits: true,
        orbit_subdivisions: 100,
        show_nodes: false,
        show_peri_and_apo_apsis: false,
        show_position_and_velocity: false,
        draw_soi: true,
        draw_axis: true,
        axis_scale: 10000.0,
        distance_scaling: 1e-6,
        velocity_scaling: 100000.0,
        focus_mode: FocusMode::Sun,
    });

    let sphere = meshes.add(
        shape::Icosphere {
            radius: 1.0,
            subdivisions: 4,
        }
        .try_into()
        .unwrap(),
    );

    let star_material = materials.add(StandardMaterial {
        emissive: Color::YELLOW * 100.0,
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 100000.0,
            range: 100000.0,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: star_material,
            transform: Transform::from_scale(Vec3::ONE * 1.0),

            ..Default::default()
        })
        .insert(NotShadowCaster)
        .insert(Star);

    if USE_REAL_SOLAR_SYSTEM {
        spawn_solar_system(&mut commands, sphere, materials.as_mut());
    } else {
        spawn_test_system(&mut commands, sphere, materials.as_mut());
    }

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

fn spawn_test_system(
    commands: &mut Commands,
    sphere: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut planet_material = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            emissive: color,
            perceptual_roughness: 1.0,
            ..Default::default()
        })
    };

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BEIGE),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                semi_major_axis: 0.38709927 * AU,
                eccentricity: 0.20563593,
                inclination: 0.12,
                right_ascension_of_the_ascending_node: 0.84,
                argument_of_periapsis: 1.35,
                mean_anomaly_at_epoch: 4.40,
                epoch: 0.0, // Example epoch year
            },
            3.285,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Test Planet"));
}

fn spawn_solar_system(
    commands: &mut Commands,
    sphere: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut planet_material = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            emissive: color,
            perceptual_roughness: 1.0,
            ..Default::default()
        })
    };

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BEIGE),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                semi_major_axis: 0.38709927 * AU,
                eccentricity: 0.20563593,
                inclination: 0.12,
                right_ascension_of_the_ascending_node: 0.84,
                argument_of_periapsis: 1.35,
                mean_anomaly_at_epoch: 4.40,
                epoch: 0.0, // Example epoch year
            },
            3.285,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Mercury"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::ORANGE),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                semi_major_axis: 0.7233 * AU,
                eccentricity: 0.00676,
                inclination: 0.0593,
                right_ascension_of_the_ascending_node: 1.34,
                argument_of_periapsis: 2.30,
                mean_anomaly_at_epoch: 3.17,
                epoch: 0.0,
            },
            4.867e1,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Venus"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BLUE),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                eccentricity: 0.01673,
                semi_major_axis: 1.0000 * AU,
                inclination: 0.01,
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: 1.7964674,
                mean_anomaly_at_epoch: 0.0,
                epoch: 0.0,
            },
            5.972e1,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Earth"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::RED),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                eccentricity: 0.09339410,
                semi_major_axis: 1.52371034 * AU,
                inclination: 0.03232349774693498376462675303241,
                right_ascension_of_the_ascending_node:
                    0.86760317116638123268876668101569,
                argument_of_periapsis: 5.8657025501025428421251399347365,
                mean_anomaly_at_epoch: 6.2034237603634456152598740984391,
                epoch: 0.0,
            },
            0.642,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Mars"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::GREEN),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                eccentricity: 0.04854,
                semi_major_axis: 5.2025 * AU,
                inclination: 0.02267182698340634120423874308267,
                right_ascension_of_the_ascending_node:
                    1.7503907068251131326967694717172,
                argument_of_periapsis: 0.24905848425959083062701067266333,
                mean_anomaly_at_epoch: 0.59917153220965334375790304082214,
                epoch: 0.0,
            },
            1.898e4,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Jupiter"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::YELLOW_GREEN),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                eccentricity: 0.05551,
                semi_major_axis: 9.5415 * AU,
                inclination: 0.04352851154473857964847684776611,
                right_ascension_of_the_ascending_node:
                    1.9833921619663561312160821893105,
                argument_of_periapsis: 1.6207127434019344451313392476185,
                mean_anomaly_at_epoch: 0.8740608893987602521233843368591,
                epoch: 0.0,
            },
            5.683e3,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Saturn"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::ALICE_BLUE),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                eccentricity: 0.04686,
                semi_major_axis: 19.188 * AU,
                inclination: 0.01349139511791616762962012964042,
                right_ascension_of_the_ascending_node:
                    1.2908455147750061550927616923742,
                argument_of_periapsis: 3.0094712292138224894895199921049,
                mean_anomaly_at_epoch: 5.4838245097661835306942363945912,
                epoch: 0.0,
            },
            8.681e2,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Uranus"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::MIDNIGHT_BLUE),
            ..Default::default()
        })
        .insert(Planet::new_from_orbit(
            KeplerianElements {
                eccentricity: 0.00895,
                semi_major_axis: 30.070 * AU,
                inclination: 0.03089232776029963351154932660225,
                right_ascension_of_the_ascending_node:
                    2.3001694212033269494277320637911,
                argument_of_periapsis: 0.81471969483095304650797885073048,
                mean_anomaly_at_epoch: 5.3096406504171494389172520558961,
                epoch: 0.0,
            },
            1.024e3,
            STAR_MASS,
            BASE_TOLERANCE,
        ))
        .insert(Name::new("Neptune"));
}

fn mass2radius(state: &State, mass: f32) -> f32 {
    mass * state.distance_scaling
}
