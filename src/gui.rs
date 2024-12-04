use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::ParamsUniform;



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
) {
    // let mut radius = params.radius;

    egui::SidePanel::left("control_panel")
        .resizable(true)
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Circle");
            ui.group(|ui| {
                ui.label("radius");
                ui.add(egui::Slider::new(&mut params.radius, 0.0..=1.).text("radius"));
                ui.add(egui::Slider::new(&mut params.noise_amplitude, 0.0..=5.).text("amplitude"));
                ui.add(egui::Slider::new(&mut params.noise_scale, 0.0..=2.).text("scale"));
                

            });

        });
}