use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource, render_resource::*, storage::ShaderStorageBuffer
    },
    utils::HashMap,
};

use bevy_egui::egui::Color32;
use crate::{data_structures::ShaderConfig, gradient_editor};

#[derive(Resource, ExtractResource, Clone)]
pub struct ParamsChanged(pub bool);
impl Default for ParamsChanged {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Resource, ExtractResource, Clone)]
pub struct Gradients {
    pub gradient: gradient_editor::Gradient,
}

impl Default for Gradients {
    fn default() -> Self {
        Self {
            gradient: gradient_editor::Gradient {
                interpolation_method: gradient_editor::InterpolationMethod::Linear,
                stops: vec![
                    (0., Color32::BLUE.into()),
                    (0.5, Color32::GREEN.into()),
                    (1., Color32::RED.into()),
                ],
            },
        }
    }
}

#[derive(Resource, ExtractResource, Clone)]
pub struct ImageBufferContainer {
    pub tex_buffer_a1: Handle<Image>,
    pub tex_buffer_b1: Handle<Image>,
    pub tex_buffer_a2: Handle<Image>,
    pub tex_buffer_b2: Handle<Image>,
    pub tex_buffer_a3: Handle<Image>,
    pub tex_buffer_b3: Handle<Image>,
    pub result: Handle<Image>,
    pub grid_buffer_a: Handle<ShaderStorageBuffer>,
    pub grid_buffer_b: Handle<ShaderStorageBuffer>,
    pub strip_buffer_a: Handle<ShaderStorageBuffer>,
    pub strip_buffer_b: Handle<ShaderStorageBuffer>,
    pub grad_texture: Handle<Image>,
}

#[derive(Resource)]
pub struct GpuBufferBindGroups {
    pub bind_groups: Vec<BindGroup>,
    pub final_pass_a: BindGroup,
    pub final_pass_b: BindGroup,
    pub uniform_buffer: Buffer,
}

#[derive(Resource)]
pub struct BindGroupSelection {
    // node_bind_groups: Vec<Selector>, // Index of bind group to use for each node
    pub selectors: HashMap<u32, Vec<u32>>,
    pub final_pass: u32,
}

#[derive(Resource, Clone, ExtractResource)]
pub struct ShaderConfigHolder {
    pub shader_configs: Vec<ShaderConfig>,
}