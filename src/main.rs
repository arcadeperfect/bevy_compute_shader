
use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        render_resource::*,
        renderer::RenderQueue,
    },
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

fn main() {
    App::new()
        .insert_resource(ParamsUniform::default())
        .insert_resource(Gradients::default())
        .add_systems(Startup, setup)
        .add_plugins((
            DefaultPlugins,
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
    commands.spawn((Camera2d::default(), CameraController::default()));
}


#[derive(Debug, Clone)]
struct ShaderConfig {
    // shader_handle: Handle<Shader>,
    shader_path: &'static str,
    iterations: u32,
}

#[derive(Copy, Clone, Pod, Zeroable, ShaderType)]
#[repr(C)]
struct DataGrid {
    // This creates a 10x20 grid
    floats: [[[f32; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
    ints: [[[i32; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
}
