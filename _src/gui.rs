use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

// use crate::gradient_editor::{gradient_editor, Gradient};

use crate::{ParamsUniform, ShaderConfigurator};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(bevy_egui::EguiPlugin);
        // app.add_systems(Update, ui_system);
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut params: ResMut<ParamsUniform>,
    mut shader_configurator: ResMut<ShaderConfigurator>,
    // mut gradients: ResMut<Gradients>,
) {
    // let mut radius = params.radius;



    // let mut g = Gradient::default();

    // egui::SidePanel::left("control_panel")
    //     .resizable(false)
    //     .default_width(600.0)
    //     .show(contexts.ctx_mut(), |ui| {
    //         ui.heading("noiseeee");
    //         ui.group(|ui| {
    //             ui.label("radius");
    //             ui.add(egui::Slider::new(&mut params.radius, 0.0..=0.5).text("radius"));
    //             ui.add(egui::Slider::new(&mut params.noise_amplitude, 0.0..=5.).text("amplitude"));
    //             ui.add(egui::Slider::new(&mut params.noise_freq, 0.0..=1.).text("frequency"));
    //             ui.add(egui::Slider::new(&mut params.noise_offset, 0.0..=20.).text("offset"));
    //             ui.add(egui::Slider::new(&mut params.noise_lacunarity, 0.0..=4.).text("lacunarity"));
    //             ui.add(egui::Slider::new(&mut params.power_bias, 0.0..=6.).text("power bias"));
    //             ui.add(egui::Slider::new(&mut params.flatness, 0.0..=6.).text("flatness"));
    //             ui.add(egui::Slider::new(&mut params.steepness, 0.0..=6.).text("steepness"));
    //             ui.add(egui::Slider::new(&mut params.mix, 0.0..=1.).text("mix"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_1_amount_1, 0.0..=0.2).text("warp 1 amount 1"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_1_scale_1, 1.0..=20.).text("warp 1 scale 1"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_1_amount_2, 0.0..=0.03).text("warp 1 amount 2"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_1_scale_2, 10.0..=70.).text("warp 1 scale 2"));
    //             ui.horizontal(|ui| {
    //                 ui.label("warp iterations");
    //                 ui.add(
    //                     egui::DragValue::new(&mut shader_configurator.shader_configs[1].iterations)
    //                     .range(0..=50),
    //                 );
    //             });
    //             ui.add(egui::Slider::new(&mut params.noise_weight, 0.0..=1.).text("noise weight"));
    //             ui.horizontal(|ui| {
    //                 ui.label("ca iterations");
    //                 ui.add(
    //                     egui::DragValue::new(&mut shader_configurator.shader_configs[3].iterations)
    //                     .range(0..=100),
    //                 );
    //             });
    //             ui.add(egui::Slider::new(&mut params.ca_thresh, 0.0..=1.).text("thresh"));
    //             ui.add(egui::Slider::new(&mut params.ca_search_radius, 0.1..=6.).text("search radius"));
    //             ui.add(egui::Slider::new(&mut params.ca_edge_pow, 0.1..=6.).text("edge pow"));
    //             ui.add(egui::Slider::new(&mut params.edge_suppress_mix, 0.0..=1.).text("edge mix"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_2_amount_1, 0.0..=0.2).text("warp 2 amount 1"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_2_scale_1, 1.0..=20.).text("warp 2 scale 1"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_2_amount_2, 0.0..=0.03).text("warp 2 amount 1"));
    //             ui.add(egui::Slider::new(&mut params.domain_warp_2_scale_2, 10.0..=70.).text("warp 2 scale 1"));
                
                
    //             ui.add(egui::Slider::new(&mut params.misc_f, 10.0..=70.).text("misc f"));
    //             ui.add(egui::Slider::new(&mut params.misc_i, 10..=70).text("misc i"));


    //             // gradient_editor(ui, &mut gradients.gradient);

    //             // let i = egui_colorgradient::InterpolationMethod::Linear;
    //             // let g = egui_colorgradient::Gradient::default();
    //             // let z = egui_colorgradient::gradient_editor(ui, &mut g);
    //         });
    //     });
}
