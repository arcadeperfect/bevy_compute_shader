use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::gradient_editor::{gradient_editor, Gradient};

use crate::{Gradients, ParamsChanged, ParamsUniform, ShaderConfigHolder};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin);
        app.add_systems(Update, ui_system);
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut params: ResMut<ParamsUniform>,
    mut configs: ResMut<ShaderConfigHolder>,
    mut gradients: ResMut<Gradients>,
    mut changed: ResMut<ParamsChanged>,
) {
    let mut old_params: ParamsUniform = params.clone();

    let mut g = Gradient::default();

    egui::SidePanel::left("control_panel")
        .resizable(false)
        .default_width(600.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("noiseeee");
            ui.group(|ui| {
                ui.label("radius");
                egui::CollapsingHeader::new("Noise Parameters")
                    .default_open(true) // Optional: starts expanded
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(&mut old_params.radius, 0.0..=1.).text("radius"));
                        ui.add(
                            egui::Slider::new(&mut old_params.noise_amplitude, 0.0..=5.)
                                .text("amplitude"),
                        );
                        ui.add(
                            egui::Slider::new(&mut old_params.noise_freq, 0.0..=1.)
                                .text("frequency"),
                        );
                        ui.add(
                            egui::Slider::new(&mut old_params.noise_offset, 0.0..=20.)
                                .text("offset"),
                        );
                        ui.add(
                            egui::Slider::new(&mut old_params.noise_octaves, 1..=20)
                                .text("octaves"),
                        );
                        ui.add(
                            egui::Slider::new(&mut old_params.noise_lacunarity, 0.0..=4.)
                                .text("lacunarity"),
                        );
                    });
                ui.add(egui::Slider::new(&mut old_params.power_bias, 0.0..=6.).text("power bias"));
                ui.add(egui::Slider::new(&mut old_params.flatness, 0.0..=1.).text("flatness"));
                ui.add(egui::Slider::new(&mut old_params.steepness, 0.0..=1.).text("steepness"));
                ui.add(egui::Slider::new(&mut old_params.mix, 0.0..=1.).text("mix"));
               
                egui::CollapsingHeader::new("Noise Parameters")
                .default_open(true) // Optional: starts expanded
                .show(ui, |ui| {
                    ui.add(
                        egui::Slider::new(&mut old_params.domain_warp_1_settings.amount_a, 0.0..=0.2)
                            .text("warp 1 amount 1"),
                    );
                    ui.add(
                        egui::Slider::new(&mut old_params.domain_warp_1_settings.scale_a, 1.0..=20.)
                            .text("warp 1 scale 1"),
                    );
                    ui.add(
                        egui::Slider::new(&mut old_params.domain_warp_1_settings.amount_b, 0.0..=0.03)
                            .text("warp 1 amount 2"),
                    );
                    ui.add(
                        egui::Slider::new(&mut old_params.domain_warp_1_settings.scale_b, 10.0..=70.)
                            .text("warp 1 scale 2"),
                    );
                });
               
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_1_amount_a, 0.0..=0.2)
                        .text("warp 1 amount 1"),
                );
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_1_scale_a, 1.0..=20.)
                        .text("warp 1 scale 1"),
                );
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_1_amount_b, 0.0..=0.03)
                        .text("warp 1 amount 2"),
                );
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_1_scale_b, 10.0..=70.)
                        .text("warp 1 scale 2"),
                );
                if configs.shader_configs.len() > 1 {
                    ui.horizontal(|ui| {
                        ui.label("warp iterations");
                        ui.add(
                            egui::DragValue::new(&mut configs.shader_configs[1].iterations)
                                .range(0..=50),
                        );
                    });
                }

                ui.add(
                    egui::Slider::new(&mut old_params.noise_weight, 0.0..=1.).text("noise weight"),
                );
                if configs.shader_configs.len() > 3 {
                    ui.horizontal(|ui| {
                        ui.label("ca iterations");
                        ui.add(
                            egui::DragValue::new(&mut configs.shader_configs[4].iterations)
                                .range(0..=100),
                        );
                    });
                }
                ui.add(egui::Slider::new(&mut old_params.ca_thresh, 0.0..=1.).text("thresh"));
                ui.add(
                    egui::Slider::new(&mut old_params.ca_search_radius, 0.1..=6.)
                        .text("search radius"),
                );
                ui.add(egui::Slider::new(&mut old_params.ca_edge_pow, 0.1..=6.).text("edge pow"));
                ui.add(
                    egui::Slider::new(&mut old_params.edge_suppress_mix, 0.0..=1.).text("edge mix"),
                );
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_2_amount_a, 0.0..=0.2)
                        .text("warp 2 amount 1"),
                );
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_2_scale_a, 1.0..=20.)
                        .text("warp 2 scale 1"),
                );
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_2_amount_b, 0.0..=0.03)
                        .text("warp 2 amount 1"),
                );
                ui.add(
                    egui::Slider::new(&mut old_params.domain_warp_2_scale_b, 10.0..=70.)
                        .text("warp 2 scale 1"),
                );

                ui.add(egui::Slider::new(&mut old_params.misc_f, 0.0..=1.).text("misc f"));
                ui.add(egui::Slider::new(&mut old_params.misc_i, 1..=2000).text("misc i"));

                // gradient_editor(ui, &mut gradients.gradient);

                // let i = egui_colorgradient::InterpolationMethod::Linear;
                // let g = egui_colorgradient::Gradient::default();
                // let z = egui_colorgradient::gradient_editor(ui, &mut g);

                if old_params != *params {
                    *params = old_params.clone();
                    changed.0 = true;
                    // println!("a");
                }
            });
        });
}
