use std::f32::consts::PI;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_egui::egui::{DragValue, Ui};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use keplerian_elements::utils::{yup2zup, zup2yup};
use keplerian_elements::{KeplerianElements, StateVectors};
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
        .add_system(draw_axis)
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
    velocity_scale: f32,

    draw_axis: bool,
    axis_scale: f32,
}

#[derive(Component)]
struct Planet {
    orbit: OrbitalRepresentation,
    mass: f32,
}

enum OrbitalRepresentation {
    Keplerian(KeplerianElements),
    StateVectors(StateVectors),
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

                    if ui.button("Flip representation").clicked() {
                        planet.orbit = match &planet.orbit {
                            OrbitalRepresentation::Keplerian(keplerian) => {
                                OrbitalRepresentation::StateVectors(
                                    keplerian.state_vectors_at_epoch(
                                        state.star_mass,
                                        state.epoch,
                                        state.tolerance,
                                    ),
                                )
                            }
                            OrbitalRepresentation::StateVectors(sv) => {
                                OrbitalRepresentation::Keplerian(
                                    KeplerianElements::state_vectors_to_orbit(
                                        sv.clone(),
                                        state.star_mass,
                                        state.epoch,
                                    ),
                                )
                            }
                        }
                    }

                    match &mut planet.orbit {
                        OrbitalRepresentation::Keplerian(orbit) => {
                            value_slider_min_max(
                                ui,
                                "Semi major axis",
                                &mut orbit.semi_major_axis,
                                f32::MIN,
                                f32::MAX,
                            );
                            value_slider(ui, "Eccentricity", &mut orbit.eccentricity);
                            value_slider(ui, "Inclination", &mut orbit.inclination);
                            value_slider(
                                ui,
                                "Longitude of ascending node",
                                &mut orbit.right_ascension_of_the_ascending_node,
                            );
                            value_slider(
                                ui,
                                "Argument of periapsis",
                                &mut orbit.argument_of_periapsis,
                            );
                            value_slider(ui, "Mean anomaly", &mut orbit.mean_anomaly_at_epoch);
                            value_slider(ui, "Epoch", &mut orbit.epoch);

                            ui.label("Readouts:");
                            ui.label(format!(
                                "True anomaly: {}",
                                orbit.true_anomaly_at_epoch(
                                    state.star_mass,
                                    state.epoch,
                                    state.tolerance
                                )
                            ));
                        }
                        OrbitalRepresentation::StateVectors(sv) => {
                            ui.label("Position");

                            let mut p = zup2yup(sv.position);

                            value_slider(ui, "X", &mut p.x);
                            value_slider(ui, "Y", &mut p.y);
                            value_slider(ui, "Z", &mut p.z);

                            sv.position = yup2zup(p);

                            ui.label("Velocity");

                            let mut v = zup2yup(sv.velocity * state.velocity_scale);

                            value_slider(ui, "Vx", &mut v.x);
                            value_slider(ui, "Vy", &mut v.y);
                            value_slider(ui, "Vz", &mut v.z);

                            sv.velocity = yup2zup(v / state.velocity_scale);
                        }
                    }
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
            if state.draw_orbits {
                ui.checkbox(&mut state.show_nodes, "Show nodes");
                ui.checkbox(
                    &mut state.show_peri_and_apo_apsis,
                    "Show peri and apo apsis",
                );

                ui.checkbox(
                    &mut state.show_position_and_velocity,
                    "Show position & velocity",
                );
                if state.show_position_and_velocity {
                    value_slider(ui, "Velocity scale", &mut state.velocity_scale);
                }

                value_slider_u32(ui, "Orbit subdivisions", &mut state.orbit_subdivisions);
            }

            ui.checkbox(&mut state.draw_axis, "Draw axis");
            if state.draw_axis {
                value_slider(ui, "Axis scale", &mut state.axis_scale);
            }
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

fn value_slider_u32(ui: &mut Ui, name: &str, value: &mut u32) {
    ui.horizontal(|ui| {
        ui.label(name);
        ui.add(
            DragValue::new(value)
                .speed(1)
                .clamp_range(u32::MIN..=u32::MAX),
        );
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
        tolerance: 0.01,
        star_mass: 4.0,
        epoch: 0.0,
        epoch_scale: 1000.0,
        update_epoch: true,
        draw_orbits: true,
        orbit_subdivisions: 100,
        show_nodes: true,
        show_peri_and_apo_apsis: true,
        show_position_and_velocity: true,
        velocity_scale: 10_000_000.00,
        draw_axis: true,
        axis_scale: 1000.0,
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

    // commands
    //     .spawn(PbrBundle {
    //         mesh: sphere.clone(),
    //         material: planet_material(Color::BEIGE),
    //         ..Default::default()
    //     })
    //     .insert(Planet {
    //         orbit: Orbit {
    //             semi_major_axis: 7.0,
    //             eccentricity: 0.12,
    //             inclination: 0.12,
    //             longitude_of_ascending_node: 0.0,
    //             argument_of_periapsis: 0.0,
    //             mean_anomaly_at_epoch_zero: 0.0,
    //         },
    //         mass: 0.3,
    //     })
    //     .insert(Name::new("Mercury"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BLUE),
            ..Default::default()
        })
        .insert(Planet {
            orbit: OrbitalRepresentation::Keplerian(KeplerianElements {
                semi_major_axis: 10.0,
                eccentricity: 0.01,
                inclination: 0.001,
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
                mean_anomaly_at_epoch: 0.0,
                epoch: 0.0,
            }),
            mass: 0.7,
        })
        .insert(Name::new("Earth"));

    // commands
    //     .spawn(PbrBundle {
    //         mesh: sphere.clone(),
    //         material: planet_material(Color::RED),
    //         ..Default::default()
    //     })
    //     .insert(Planet {
    //         orbit: Orbit {
    //             semi_major_axis: 20.0,
    //             eccentricity: 0.1,
    //             inclination: 0.1,
    //             longitude_of_ascending_node: 0.0,
    //             argument_of_periapsis: 0.0,
    //             mean_anomaly_at_epoch_zero: 0.0,
    //         },
    //         mass: 0.6,
    //     })
    //     .insert(Name::new("Mars"));

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
        let StateVectors { position, .. } = match &planet.orbit {
            OrbitalRepresentation::Keplerian(keplerian) => {
                keplerian.state_vectors_at_epoch(state.star_mass, state.epoch, state.tolerance)
            }
            OrbitalRepresentation::StateVectors(sv) => sv.clone(),
        };

        let position = zup2yup(position);

        transform.translation = position;
        transform.scale = Vec3::ONE * planet.mass;
    }
}

fn update_star(mut query: Query<&mut Transform, With<Star>>, state: Res<State>) {
    for mut transform in query.iter_mut() {
        transform.scale = Vec3::ONE * state.star_mass;
    }
}

fn draw_orbits(
    mut lines: ResMut<DebugLines>,
    planets: Query<&Planet>,
    state: Res<State>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    if !state.draw_orbits {
        return;
    }

    let camera = camera.single();
    let camera_position = camera.translation();

    let color = Color::RED;

    for planet in planets.iter() {
        let orbit = match &planet.orbit {
            OrbitalRepresentation::Keplerian(orbit) => orbit.clone(),
            OrbitalRepresentation::StateVectors(sv) => {
                KeplerianElements::state_vectors_to_orbit(sv.clone(), state.star_mass, state.epoch)
            }
        };

        let first_position = zup2yup(orbit.position_at_true_anomaly(0.0));
        let mut prev_position = first_position.clone();

        let step = (2.0 * PI) / state.orbit_subdivisions as f32;

        for i in 0..state.orbit_subdivisions {
            let t = i as f32 * step;

            let position = orbit.position_at_true_anomaly(t);
            let position = zup2yup(position);

            lines.line_colored(prev_position, position, 0.0, color);

            prev_position = position;
        }

        // Close the loop
        lines.line_colored(prev_position, first_position, 0.0, color);

        let mut debug_arrows = DebugArrows::new(&mut lines, camera_position);

        if state.show_position_and_velocity {
            let StateVectors { position, velocity } =
                orbit.state_vectors_at_epoch(state.star_mass, state.epoch, state.tolerance);

            let position = zup2yup(position);
            let velocity = zup2yup(velocity);

            debug_arrows.draw_arrow(Vec3::ZERO, position, color);
            debug_arrows.draw_arrow(
                position,
                position + (state.velocity_scale * velocity),
                color,
            );
        }

        if state.show_nodes {
            let normal = zup2yup(orbit.normal());

            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.ascending_node()),
                Color::YELLOW_GREEN,
            );
            debug_arrows.draw_arrow(Vec3::ZERO, zup2yup(orbit.descending_node()), Color::YELLOW);

            debug_arrows.draw_arrow(Vec3::ZERO, zup2yup(orbit.periapsis()), Color::WHITE);
            debug_arrows.draw_arrow(Vec3::ZERO, zup2yup(orbit.apoapsis()), Color::WHITE);

            debug_arrows.draw_arrow(Vec3::ZERO, 10.0 * normal, Color::GREEN);
        }
    }
}

const ARROW_WING_LENGTH: f32 = 1.0;
const ARROW_WING_ANGLE: f32 = 30.0;

fn draw_axis(mut lines: ResMut<DebugLines>, state: Res<State>) {
    if !state.draw_axis {
        return;
    }

    const ORIGIN: Vec3 = Vec3::ZERO;

    lines.line_colored(ORIGIN, ORIGIN + state.axis_scale * Vec3::X, 0.0, Color::RED);
    lines.line_colored(
        ORIGIN,
        ORIGIN + state.axis_scale * Vec3::Y,
        0.0,
        Color::GREEN,
    );
    lines.line_colored(
        ORIGIN,
        ORIGIN + state.axis_scale * Vec3::Z,
        0.0,
        Color::BLUE,
    );
}

struct DebugArrows<'a> {
    lines: &'a mut DebugLines,
    camera_position: Vec3,
}

impl<'a> DebugArrows<'a> {
    pub fn new(lines: &'a mut DebugLines, camera_position: Vec3) -> Self {
        Self {
            lines,
            camera_position,
        }
    }

    pub fn draw_arrow(&mut self, start: Vec3, end: Vec3, color: Color) {
        self.lines.line_colored(start, end, 0.0, color);

        let to_start = (start - end).normalize();
        let axis_start = closest_point(self.camera_position, start, end);
        let rot_axis = (self.camera_position - axis_start).normalize();

        let angle = deg2rad(ARROW_WING_ANGLE);
        let rot_1 = Quat::from_axis_angle(rot_axis, angle);
        let rot_2 = Quat::from_axis_angle(rot_axis, -angle);

        let wing_1 = (rot_1 * to_start) * ARROW_WING_LENGTH + end;
        let wing_2 = (rot_2 * to_start) * ARROW_WING_LENGTH + end;

        self.lines.line_colored(end, wing_1, 0.0, color);
        self.lines.line_colored(end, wing_2, 0.0, color);
    }
}

/// Finds the closest point on the line segment defined by `a` and `b` to `pos`.
/// By definition the lines given by a and b and the pos and found point must be perpendicular.
fn closest_point(pos: Vec3, a: Vec3, b: Vec3) -> Vec3 {
    let ab = b - a;
    let ap = pos - a;

    let t = ap.dot(ab) / ab.dot(ab);

    if t < 0.0 {
        a
    } else if t > 1.0 {
        b
    } else {
        a + ab * t
    }
}

fn deg2rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}
