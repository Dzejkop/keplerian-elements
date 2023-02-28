use bevy::core_pipeline::bloom::BloomSettings;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_egui::egui::{DragValue, Ui};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use keplerian_orbits::{Orbit, StateVectors};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(OrbitCameraPlugin::new(false))
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(ui)
        .add_system(update_epoch)
        .add_system(draw_orbits)
        .add_system(update_planets)
        .add_system(update_star)
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
}

#[derive(Component)]
struct Planet {
    orbit: Orbit,
    mass: f32,
}

#[derive(Component)]
struct Star;

fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<State>,
    mut planets: Query<(&mut Planet, &Name)>,
    mut camera: Query<&mut OrbitCameraController>,
) {
    egui::Window::new("Settings").show(egui_context.ctx_mut(), |ui| {
        ui.collapsing("Orbits", |ui| {
            for (mut planet, name) in planets.iter_mut() {
                ui.collapsing(name.as_str(), |ui| {
                    ui.label(name.to_string());

                    value_slider_min_max(
                        ui,
                        "Semi major axis",
                        &mut planet.orbit.semi_major_axis,
                        f32::EPSILON,
                        f32::MAX,
                    );
                    value_slider(ui, "Eccentricity", &mut planet.orbit.eccentricity);
                    value_slider(ui, "Inclination", &mut planet.orbit.inclination);
                    value_slider(
                        ui,
                        "Longitude of ascending node",
                        &mut planet.orbit.longitude_of_ascending_node,
                    );
                    value_slider(
                        ui,
                        "Argument of periapsis",
                        &mut planet.orbit.argument_of_periapsis,
                    );
                });
            }
        });

        ui.collapsing("State", |ui| {
            value_slider_min_max(ui, "Tolerance", &mut state.tolerance, f32::EPSILON, 100.0);
            value_slider(ui, "Mass", &mut state.star_mass);
            value_slider(ui, "Epoch", &mut state.epoch);
            value_slider(ui, "Epoch scale", &mut state.epoch_scale);
            ui.checkbox(&mut state.update_epoch, "Update Epoch");
            ui.checkbox(&mut state.draw_orbits, "Draw orbits");
        });

        if let Ok(mut camera) = camera.get_single_mut() {
            ui.collapsing("Camera", |ui| {
                ui.label("Mouse rotate sensitivity");
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut camera.mouse_rotate_sensitivity.x).speed(0.01));
                    ui.label("y");
                    ui.add(DragValue::new(&mut camera.mouse_rotate_sensitivity.y).speed(0.01));
                });

                ui.label("Mouse translate sensitivity");
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(DragValue::new(&mut camera.mouse_translate_sensitivity.x).speed(0.01));
                    ui.label("y");
                    ui.add(DragValue::new(&mut camera.mouse_translate_sensitivity.y).speed(0.01));
                });
            });
        }
    });
}

fn value_slider(ui: &mut Ui, name: &str, value: &mut f32) {
    value_slider_min_max(ui, name, value, f32::MIN, f32::MAX)
}

fn value_slider_min_max(ui: &mut Ui, name: &str, value: &mut f32, min: f32, max: f32) {
    ui.horizontal(|ui| {
        ui.label(name);
        ui.add(DragValue::new(value).speed(0.01).clamp_range(min..=max));
    });
}

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
        tolerance: 0.1,
        star_mass: 4.0,
        epoch: 0.0,
        epoch_scale: 1000.0,
        update_epoch: true,
        draw_orbits: true,
    });

    let sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 1.0,
        subdivisions: 4,
    }));

    let star_material = materials.add(StandardMaterial {
        emissive: Color::YELLOW * 100.0,
        ..Default::default()
    });

    let mut planet_material = |color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 1.0,
            ..Default::default()
        })
    };

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

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BEIGE),
            ..Default::default()
        })
        .insert(Planet {
            orbit: Orbit {
                semi_major_axis: 7.0,
                eccentricity: 0.12,
                inclination: 0.12,
                longitude_of_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
            },
            mass: 0.3,
        })
        .insert(Name::new("Mercury"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BLUE),
            ..Default::default()
        })
        .insert(Planet {
            orbit: Orbit {
                semi_major_axis: 10.0,
                eccentricity: 0.01,
                inclination: 0.001,
                longitude_of_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
            },
            mass: 0.7,
        })
        .insert(Name::new("Earth"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::RED),
            ..Default::default()
        })
        .insert(Planet {
            orbit: Orbit {
                semi_major_axis: 20.0,
                eccentricity: 0.1,
                inclination: 0.1,
                longitude_of_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
            },
            mass: 0.6,
        })
        .insert(Name::new("Mars"));

    commands
        .spawn(Camera3dBundle::default())
        .insert(BloomSettings {
            intensity: 0.9,
            threshold: 0.7,
            ..default()
        })
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

fn update_epoch(time: Res<Time>, mut state: ResMut<State>) {
    if state.update_epoch {
        state.epoch += state.epoch_scale * time.delta_seconds();
    }
}

fn update_planets(mut query: Query<(&mut Transform, &Planet)>, state: Res<State>) {
    for (mut transform, planet) in query.iter_mut() {
        let StateVectors { position, .. } =
            planet
                .orbit
                .state_vectors_at_epoch(state.star_mass, state.epoch, state.tolerance);

        transform.translation = position;
        transform.scale = Vec3::ONE * planet.mass;
    }
}

fn update_star(mut query: Query<&mut Transform, With<Star>>, state: Res<State>) {
    for mut transform in query.iter_mut() {
        transform.scale = Vec3::ONE * state.star_mass;
    }
}

fn draw_orbits(mut lines: ResMut<DebugLines>, planets: Query<&Planet>, state: Res<State>) {
    if !state.draw_orbits {
        return;
    }

    let color = Color::RED;

    for planet in planets.iter() {
        let mut t = 0.0;
        let period = planet.orbit.period(state.star_mass);
        let step = period / 100.0;

        let StateVectors {
            position: first_position,
            ..
        } = planet
            .orbit
            .state_vectors_at_epoch(state.star_mass, t, state.tolerance);

        let mut prev_position = first_position.clone();

        t += step;

        while t < period {
            let StateVectors { position, .. } =
                planet
                    .orbit
                    .state_vectors_at_epoch(state.star_mass, t, state.tolerance);

            lines.line_colored(prev_position, position, 0.0, color);

            prev_position = position;

            t += step;
        }

        // Close the loop
        lines.line_colored(prev_position, first_position, 0.0, color);
    }
}
