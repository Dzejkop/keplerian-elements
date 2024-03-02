use bevy::prelude::*;
use bevy_egui::egui::{ComboBox, DragValue, Ui};
use bevy_egui::{egui, EguiContexts};
use keplerian_elements::utils::{yup2zup, zup2yup};
use smooth_bevy_cameras::controllers::orbit::OrbitCameraController;

use super::{FocusMode, Planet, State};

#[derive(Debug, Clone, Default)]
pub struct UiState {
    selected_planet: Option<usize>,
    settings_visible: bool,
    about_visible: bool,
    focus_visible: bool,
}

pub fn render(
    mut ui_state: Local<UiState>,
    mut egui_context: EguiContexts,
    mut state: ResMut<State>,
    mut planets: Query<(&mut Planet, &Name)>,
    mut camera: Query<&mut OrbitCameraController>,
    camera_transform: Query<&GlobalTransform, With<OrbitCameraController>>,
) {
    let ctx = egui_context.ctx_mut();

    egui::TopBottomPanel::top("Top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Settings").clicked() {
                ui_state.settings_visible = !ui_state.settings_visible;
            }

            if ui.button("About").clicked() {
                ui_state.about_visible = !ui_state.about_visible;
            }

            if ui.button("Focus").clicked() {
                ui_state.focus_visible = !ui_state.focus_visible;
            }
        });
    });

    egui::TopBottomPanel::bottom("Bototm").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Ok(camera_transform) = camera_transform.get_single() {
                let translation = camera_transform.translation();
                ui.label(format!("Camera Position: {translation}"));
            }

            let epoch_seconds = state.epoch;
            let (y, m, d) = epoch_years_months_days(epoch_seconds as f64);

            ui.label(format!("Epoch: {y:.0}Y {m:.0}M {d:.0}D"));
        });
    });

    egui::SidePanel::left("Left").show(ctx, |ui| {
        ui.heading("Planets:");
        for (idx, (_planet, name)) in planets.iter().enumerate() {
            let selected = ui_state.selected_planet == Some(idx);

            if ui.selectable_label(selected, name.as_str()).clicked() {
                if ui_state.selected_planet == Some(idx) {
                    ui_state.selected_planet = None;
                } else {
                    ui_state.selected_planet = Some(idx);
                }
            }
        }

        ui.separator();

        if let Some(selected_idx) = ui_state.selected_planet {
            let (_idx, (mut planet, name)) = planets
                .iter_mut()
                .enumerate()
                .find(|(idx, _)| *idx == selected_idx)
                .unwrap();

            ui.heading(name.to_string());

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
                sv.velocity * state.distance_scaling * state.velocity_scaling,
            );

            value_slider(ui, "Vx", &mut v.x);
            value_slider(ui, "Vy", &mut v.y);
            value_slider(ui, "Vz", &mut v.z);

            sv.velocity =
                yup2zup(v / (state.distance_scaling * state.velocity_scaling));

            if ui.button("Focus").clicked() {
                state.focus_mode = FocusMode::Planet(name.to_string());
            }
        }
    });

    egui::Window::new("Settings")
        .open(&mut ui_state.settings_visible)
        .show(ctx, |ui| {
            ui.heading("State");
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

            if let Ok(mut camera) = camera.get_single_mut() {
                ui.heading("Camera");

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
            }
        });

    egui::Window::new("About").open(&mut ui_state.about_visible).show(ctx, |ui| {
        ui.heading("Hello!");

        ui.label("This is a basic orbital simulation of the solar system.");

        ui.label("Everything is **mostly** to scale. I've also tried to replicate the orbital elements of each planet as closely as I could");

        ui.heading("Controls");
        ui.label("Scroll to zoom in & out");
        ui.label("Hold Ctrl and drag the mouse to rotate the viewport");
        ui.label("You can use the right click and drag, but it's not very efficient");

        ui.label("Use the focus window to focus on a different celestial object");
    });

    egui::Window::new("Focus")
        .open(&mut ui_state.focus_visible)
        .show(ctx, |ui| {
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
                            state.focus_mode =
                                FocusMode::Planet(name.to_string());
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

fn epoch_years_months_days(epoch_seconds: f64) -> (u32, u32, u32) {
    const SECONDS_IN_YEAR: f64 = 365.0 * 24.0 * 60.0 * 60.0;
    const SECONDS_IN_MONTH: f64 = 30.0 * 24.0 * 60.0 * 60.0;
    const SECONDS_IN_DAY: f64 = 24.0 * 60.0 * 60.0;

    let years = (epoch_seconds / SECONDS_IN_YEAR) as u32;
    let months = ((epoch_seconds % SECONDS_IN_YEAR) / SECONDS_IN_MONTH) as u32;
    let days = ((epoch_seconds % SECONDS_IN_MONTH) / SECONDS_IN_DAY) as u32;

    (years, months, days)
}
