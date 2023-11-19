use std::f32::consts::PI;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_egui::egui::{ComboBox, DragValue, Ui};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use keplerian_elements::constants::AU;
use keplerian_elements::utils::{yup2zup, zup2yup};
use keplerian_elements::{KeplerianElements, StateVectors};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::{LookTransform, LookTransformPlugin};

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
        .add_system(draw_soi)
        .add_system(update_camera_focus)
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

    draw_soi: bool,

    draw_axis: bool,
    axis_scale: f32,

    distance_scaling: f32,
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

                    value_slider(ui, "Mass", &mut planet.mass);

                    // --- Elements ---
                    ui.collapsing("Orbital Elements", |ui| {
                        let orbit = &mut planet.orbit;
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

                        planet.state_vectors = planet.orbit.state_vectors_at_epoch(
                            state.star_mass,
                            state.epoch,
                            state.tolerance,
                        );
                    });

                    // --- State Vectors ---
                    ui.collapsing("State Vectors", |ui| {
                        let sv = &mut planet.state_vectors;
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

                        let sv = sv.clone();
                        planet.orbit = KeplerianElements::state_vectors_to_orbit(
                            sv,
                            state.star_mass,
                            state.epoch,
                        );
                    });
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

            ui.checkbox(&mut state.draw_soi, "Draw SOI");

            ui.checkbox(&mut state.draw_axis, "Draw axis");
            if state.draw_axis {
                value_slider(ui, "Axis scale", &mut state.axis_scale);
            }

            value_slider_min_max_with_speed(
                ui,
                "Distance scaling",
                &mut state.distance_scaling,
                0.000001,
                1.0,
                0.0000001,
            );
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

    egui::Window::new("About").show(egui_context.ctx_mut(), |ui| {
        ui.heading("Hello!");

        ui.label("This is a basic orbital simulation of the solar system.");

        ui.label("Everything is **mostly** to scale. I've also tried to replicate the orbital elements of each planet as closely as I could");

        ui.heading("Controls");
        ui.label("Scroll to zoom in & out");
        ui.label("Hold Ctrl and drag the mouse to rotate the viewport");
        ui.label("You can use the right click and drag, but it's not very efficient");

        ui.label("Use the focus window to focus on a different celestial object");
    });

    egui::Window::new("Focus").show(egui_context.ctx_mut(), |ui| {
        let current = match &state.focus_mode {
            FocusMode::Sun => "Sun".to_string(),
            FocusMode::Planet(planet) => planet.clone(),
        };

        ComboBox::from_label("Choose focus")
            .selected_text(&current)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(current == "Sun".to_string(), "Sun")
                    .clicked()
                {
                    state.focus_mode = FocusMode::Sun;
                }

                for (_, name) in &planets {
                    if ui
                        .selectable_label(current == name.to_string(), &name.to_string())
                        .clicked()
                    {
                        state.focus_mode = FocusMode::Planet(name.to_string());
                    }
                }
            });
    });
}

fn value_slider(ui: &mut Ui, name: &str, value: &mut f32) {
    value_slider_min_max(ui, name, value, f32::MIN, f32::MAX)
}

fn value_slider_min_max(ui: &mut Ui, name: &str, value: &mut f32, min: f32, max: f32) {
    value_slider_min_max_with_speed(ui, name, value, min, max, 0.01);
}

fn value_slider_min_max_with_speed(
    ui: &mut Ui,
    name: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    speed: f32,
) {
    ui.horizontal(|ui| {
        ui.label(name);
        ui.add(DragValue::new(value).speed(speed).clamp_range(min..=max));
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
        // Sun Mass
        star_mass: 1.989e7,
        epoch: 0.0,
        epoch_scale: 1000.0,
        update_epoch: true,
        draw_orbits: true,
        orbit_subdivisions: 100,
        show_nodes: false,
        show_peri_and_apo_apsis: false,
        show_position_and_velocity: false,
        velocity_scale: 10_000_000.00,
        draw_soi: true,
        draw_axis: true,
        axis_scale: 1000.0,
        distance_scaling: 1e-6,
        focus_mode: FocusMode::Sun,
    });

    let sphere = meshes.add(Mesh::from(shape::Icosphere {
        radius: 1.0,
        subdivisions: 4,
    }));

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

    spawn_solar_system(&mut commands, sphere, materials.as_mut());

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
        .insert(Planet {
            orbit: KeplerianElements {
                semi_major_axis: 0.38709927 * AU,
                eccentricity: 0.20563593,
                inclination: 0.12,
                right_ascension_of_the_ascending_node: 0.84,
                argument_of_periapsis: 1.35,
                mean_anomaly_at_epoch: 4.40,
                epoch: 0.0, // Example epoch year
            },
            state_vectors: StateVectors::default(),
            mass: 3.285,
        })
        .insert(Name::new("Mercury"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::ORANGE),
            ..Default::default()
        })
        .insert(Planet {
            orbit: KeplerianElements {
                semi_major_axis: 0.7233 * AU,
                eccentricity: 0.00676,
                inclination: 0.0593,
                right_ascension_of_the_ascending_node: 1.34,
                argument_of_periapsis: 2.30,
                mean_anomaly_at_epoch: 3.17,
                epoch: 0.0,
            },
            state_vectors: StateVectors::default(),
            mass: 4.867e1,
        })
        .insert(Name::new("Venus"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::BLUE),
            ..Default::default()
        })
        .insert(Planet {
            orbit: KeplerianElements {
                eccentricity: 0.01673,
                semi_major_axis: 1.0000 * AU,
                inclination: 0.01,
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: 1.7964674,
                mean_anomaly_at_epoch: 0.0,
                epoch: 0.0,
            },
            state_vectors: StateVectors::default(),
            mass: 5.972e1,
        })
        .insert(Name::new("Earth"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::RED),
            ..Default::default()
        })
        .insert(Planet {
            orbit: KeplerianElements {
                eccentricity: 0.09339410,
                semi_major_axis: 1.52371034 * AU,
                inclination: 0.03232349774693498376462675303241,
                right_ascension_of_the_ascending_node: 0.86760317116638123268876668101569,
                argument_of_periapsis: 5.8657025501025428421251399347365,
                mean_anomaly_at_epoch: 6.2034237603634456152598740984391,
                epoch: 0.0,
            },
            state_vectors: StateVectors::default(),
            mass: 0.642,
        })
        .insert(Name::new("Mars"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::GREEN),
            ..Default::default()
        })
        .insert(Planet {
            orbit: KeplerianElements {
                eccentricity: 0.04854,
                semi_major_axis: 5.2025 * AU,
                inclination: 0.02267182698340634120423874308267,
                right_ascension_of_the_ascending_node: 1.7503907068251131326967694717172,
                argument_of_periapsis: 0.24905848425959083062701067266333,
                mean_anomaly_at_epoch: 0.59917153220965334375790304082214,
                epoch: 0.0,
            },
            state_vectors: StateVectors::default(),
            mass: 1.898e4,
        })
        .insert(Name::new("Jupiter"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::YELLOW_GREEN),
            ..Default::default()
        })
        .insert(Planet {
            orbit: KeplerianElements {
                eccentricity: 0.05551,
                semi_major_axis: 9.5415 * AU,
                inclination: 0.04352851154473857964847684776611,
                right_ascension_of_the_ascending_node: 1.9833921619663561312160821893105,
                argument_of_periapsis: 1.6207127434019344451313392476185,
                mean_anomaly_at_epoch: 0.8740608893987602521233843368591,
                epoch: 0.0,
            },
            state_vectors: StateVectors::default(),
            mass: 5.683e3,
        })
        .insert(Name::new("Saturn"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::ALICE_BLUE),
            ..Default::default()
        })
        .insert(Planet {
            orbit: KeplerianElements {
                eccentricity: 0.04686,
                semi_major_axis: 19.188 * AU,
                inclination: 0.01349139511791616762962012964042,
                right_ascension_of_the_ascending_node: 1.2908455147750061550927616923742,
                argument_of_periapsis: 3.0094712292138224894895199921049,
                mean_anomaly_at_epoch: 5.4838245097661835306942363945912,
                epoch: 0.0,
            },
            state_vectors: StateVectors::default(),
            mass: 8.681e2,
        })
        .insert(Name::new("Uranus"));

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: planet_material(Color::MIDNIGHT_BLUE),
            ..Default::default()
        })
        .insert(Planet {
            orbit: KeplerianElements {
                eccentricity: 0.00895,
                semi_major_axis: 30.070 * AU,
                inclination: 0.03089232776029963351154932660225,
                right_ascension_of_the_ascending_node: 2.3001694212033269494277320637911,
                argument_of_periapsis: 0.81471969483095304650797885073048,
                mean_anomaly_at_epoch: 5.3096406504171494389172520558961,
                epoch: 0.0,
            },
            state_vectors: StateVectors::default(),
            mass: 1.024e3,
        })
        .insert(Name::new("Neptune"));
}

fn update_epoch(time: Res<Time>, mut state: ResMut<State>) {
    if state.update_epoch {
        state.epoch += state.epoch_scale * time.delta_seconds();
    }
}

fn update_planets(mut query: Query<(&mut Transform, &mut Planet)>, state: Res<State>) {
    for (mut transform, mut planet) in query.iter_mut() {
        planet.state_vectors =
            planet
                .orbit
                .state_vectors_at_epoch(state.star_mass, state.epoch, state.tolerance);

        let position = zup2yup(planet.state_vectors.position);

        transform.translation = position * state.distance_scaling;
        transform.scale = Vec3::ONE * mass2radius(state.as_ref(), planet.mass);
    }
}

fn update_star(mut query: Query<&mut Transform, With<Star>>, state: Res<State>) {
    for mut transform in query.iter_mut() {
        transform.scale = Vec3::ONE * mass2radius(state.as_ref(), state.star_mass);
    }
}

fn update_camera_focus(
    mut look_transform: Query<&mut LookTransform>,
    state: Res<State>,
    planets: Query<(&GlobalTransform, &Name), With<Planet>>,
) {
    let mut look = look_transform.single_mut();

    match &state.focus_mode {
        FocusMode::Sun => {
            look.target = Vec3::ZERO;
        }
        FocusMode::Planet(focused_name) => {
            for (transform, name) in planets.iter() {
                if focused_name == name.as_ref() {
                    look.target = transform.translation();
                }
            }
        }
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
        let orbit = &planet.orbit;

        let first_position = zup2yup(orbit.position_at_true_anomaly(0.0)) * state.distance_scaling;
        let mut prev_position = first_position.clone();

        let step = (2.0 * PI) / state.orbit_subdivisions as f32;

        for i in 0..state.orbit_subdivisions {
            let t = i as f32 * step;

            let position = orbit.position_at_true_anomaly(t);
            let position = zup2yup(position) * state.distance_scaling;

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

            debug_arrows.draw_arrow(Vec3::ZERO, position * state.distance_scaling, color);
            debug_arrows.draw_arrow(
                position,
                position + (state.velocity_scale * velocity),
                color,
            );
        }

        if state.show_nodes {
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.ascending_node()) * state.distance_scaling,
                Color::YELLOW_GREEN,
            );
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.descending_node()) * state.distance_scaling,
                Color::YELLOW,
            );
        }

        if state.show_peri_and_apo_apsis {
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.periapsis()) * state.distance_scaling,
                Color::WHITE,
            );
            debug_arrows.draw_arrow(
                Vec3::ZERO,
                zup2yup(orbit.apoapsis()) * state.distance_scaling,
                Color::WHITE,
            );
        }
    }
}

fn draw_soi(
    mut lines: ResMut<DebugLines>,
    planets: Query<&Planet>,
    state: Res<State>,
    camera: Query<&GlobalTransform, With<Camera>>,
) {
    if !state.draw_soi {
        return;
    }

    let camera = camera.single();

    let camera_position = camera.translation();

    for planet in planets.iter() {
        let r = planet.state_vectors.position.length();

        let soi = keplerian_elements::astro::soi(r, planet.mass, state.star_mass)
            * state.distance_scaling;

        let pos = zup2yup(planet.state_vectors.position) * state.distance_scaling;

        let to_camera = (camera_position - pos).normalize();
        let planet_camera_radial = to_camera.cross(pos).normalize();

        let mut prev_pos = pos + planet_camera_radial * soi;
        for i in 0..=100 {
            let t = i as f32 * 2.0 * PI / 100.0;

            let rot_matrix = Mat3::from_axis_angle(to_camera, t);

            let p = rot_matrix * planet_camera_radial;
            let p = pos + p * soi;

            lines.line_colored(prev_pos, p, 0.0, Color::WHITE);

            prev_pos = p;
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

fn mass2radius(state: &State, mass: f32) -> f32 {
    mass * state.distance_scaling
}
