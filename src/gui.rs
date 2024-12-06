use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{ParamsUniform, ShaderConfigurator};

#[derive(Event)]
pub struct ParamsChanged {
    pub radius: f32,
}

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin);
        app.add_systems(Update, ui_system);
        app.add_event::<ParamsChanged>();
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    // mut param_events: EventWriter<ParamsChanged>,
    mut params: ResMut<ParamsUniform>,
    mut shader_configurator: ResMut<ShaderConfigurator>,
) {
    // let mut radius = params.radius;

    egui::SidePanel::left("control_panel")
        .resizable(false)
        .default_width(600.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("noiseeee");
            ui.group(|ui| {
                ui.label("radius");
                ui.add(egui::Slider::new(&mut params.radius, 0.0..=1.).text("radius"));
                ui.add(egui::Slider::new(&mut params.noise_amplitude, 0.0..=5.).text("amplitude"));
                ui.add(egui::Slider::new(&mut params.noise_scale, 0.0..=2.).text("scale"));
                ui.add(egui::Slider::new(&mut params.noise_offset, 0.0..=20.).text("offset"));
                ui.add(egui::Slider::new(&mut params.warp_amount, 0.0..=0.2).text("warp amount"));
                ui.add(egui::Slider::new(&mut params.warp_scale, 1.0..=20.).text("warp scale"));
                ui.horizontal(|ui| {
                    ui.label("war iterations");
                    ui.add(
                        egui::DragValue::new(&mut shader_configurator.shader_configs[1].iterations)
                            .range(0..=50),
                    );
                });
            });
        });
}
