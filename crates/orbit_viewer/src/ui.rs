use bevy::prelude::*;
use bevy_egui::egui::{ComboBox, DragValue, Ui};
use bevy_egui::{egui, EguiContexts};
use keplerian_elements::utils::{yup2zup, zup2yup};
use smooth_bevy_cameras::controllers::orbit::OrbitCameraController;

use super::{FocusMode, Planet, State};

pub fn render(
    mut egui_context: EguiContexts,
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

                    // --- State Vectors ---
                    let sv = &mut planet.state_vectors;
                    ui.label("Position");

                    let mut p = zup2yup(sv.position * state.distance_scaling);

                    value_slider(ui, "X", &mut p.x);
                    value_slider(ui, "Y", &mut p.y);
                    value_slider(ui, "Z", &mut p.z);

                    sv.position = yup2zup(p / state.distance_scaling);

                    ui.label("Velocity");

                    let mut v = zup2yup(
                        sv.velocity
                            * state.distance_scaling
                            * state.velocity_scaling,
                    );

                    value_slider(ui, "Vx", &mut v.x);
                    value_slider(ui, "Vy", &mut v.y);
                    value_slider(ui, "Vz", &mut v.z);

                    sv.velocity = yup2zup(
                        v / (state.distance_scaling * state.velocity_scaling),
                    );

                    // planet.orbit = sv.to_elements(state.star_mass, state.epoch);
                });
            }
        });

        ui.collapsing("State", |ui| {
            value_slider_min_max(
                ui,
                "Tolerance",
                &mut state.tolerance,
                f32::EPSILON,
                100.0,
            );
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

                value_slider_u32(
                    ui,
                    "Orbit subdivisions",
                    &mut state.orbit_subdivisions,
                );
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

            value_slider(ui, "Velocity scaling", &mut state.velocity_scaling);
        });

        if let Ok(mut camera) = camera.get_single_mut() {
            ui.collapsing("Camera", |ui| {
                ui.label("Mouse rotate sensitivity");
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(
                        DragValue::new(&mut camera.mouse_rotate_sensitivity.x)
                            .speed(0.01),
                    );
                    ui.label("y");
                    ui.add(
                        DragValue::new(&mut camera.mouse_rotate_sensitivity.y)
                            .speed(0.01),
                    );
                });

                ui.label("Mouse translate sensitivity");
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(
                        DragValue::new(
                            &mut camera.mouse_translate_sensitivity.x,
                        )
                        .speed(0.01),
                    );
                    ui.label("y");
                    ui.add(
                        DragValue::new(
                            &mut camera.mouse_translate_sensitivity.y,
                        )
                        .speed(0.01),
                    );
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
                        .selectable_label(
                            current == name.to_string(),
                            &name.to_string(),
                        )
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

fn value_slider_min_max(
    ui: &mut Ui,
    name: &str,
    value: &mut f32,
    min: f32,
    max: f32,
) {
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
