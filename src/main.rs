
use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        render_resource::*,
        renderer::RenderQueue,
    }, window::WindowResolution,
};
use bytemuck::{Pod, Zeroable};
use cam_controller::CameraController;
// use gradient_editor::update_gradient_texture;
use constants::*;
use parameters::ParamsUniform;
use resources::*;

mod cam_controller;
mod compute_node;
mod compute_plugin;
mod constants;
mod gradient_editor;
mod gui;
mod parameters;
mod pipeline;
mod resources;
mod bind_groups;
mod data_structures;

fn main() {
    App::new()
        .insert_resource(ParamsUniform::default())
        .insert_resource(Gradients::default())
        .add_systems(Startup, setup)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    mode: bevy::window::WindowMode::Windowed,
                    resolution: WindowResolution::new(1920., 1080.),
                    ..default()
                }),
                ..default()
            }),
            cam_controller::CameraControllerPlugin,
            compute_plugin::ComputeShaderPlugin,
            ExtractResourcePlugin::<Gradients>::default(),
            ExtractResourcePlugin::<ImageBufferContainer>::default(),
            ExtractResourcePlugin::<ParamsUniform>::default(),
            gui::GuiPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .run();
}

fn setup(mut commands: Commands) {
    // commands.spawn((Camera2d::default(), CameraController::default()));
    commands.spawn((
        Camera2d::default(),
        CameraController::default(),
        // Add or modify the Transform component
        Transform::from_xyz(0.0, 0.0, 0.0)  // Position
            .with_scale(Vec3::splat(1.5))   // Scale/Zoom
            .with_rotation(Quat::IDENTITY),  // Rotation
    ));
}


// #[derive(Debug, Clone)]
// struct ShaderConfig {
//     // shader_handle: Handle<Shader>,
//     shader_path: &'static str,
//     iterations: u32,
// }

// #[derive(Copy, Clone, Pod, Zeroable, ShaderType)]
// #[repr(C)]
// struct DataGrid {
//     // This creates a 10x20 grid
//     floats: [[[f32; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
//     ints: [[[i32; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
// }
