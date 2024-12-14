use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::ShaderType}};

#[derive(Resource, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ExtractResource, ShaderType, PartialEq)]
#[repr(C)]
pub struct ParamsUniform {
    pub dimensions: u32,

    // circle generator
    pub radius: f32,
    pub noise_seed: u32,
    pub noise_freq: f32,
    pub noise_amplitude: f32,
    pub noise_offset: f32,
    pub noise_octaves: i32,
    pub noise_lacunarity:f32,
    pub power_bias: f32,
    pub flatness: f32,
    pub steepness: f32,
    pub mix: f32,
    pub noise_warp_amount: f32,
    pub noise_warp_scale: f32,

    // domain warp 1
    pub domain_warp_1_amount_a: f32,
    pub domain_warp_1_scale_a: f32,
    pub domain_warp_1_amount_b: f32,
    pub domain_warp_1_scale_b: f32,
    
    // cellular automata
    pub noise_weight: f32,
    pub ca_thresh: f32,
    pub ca_search_radius: f32,
    pub ca_edge_pow: f32,
    pub edge_suppress_mix: f32,

    // cave domain warp
    pub domain_warp_2_amount_a: f32,
    pub domain_warp_2_scale_a: f32,
    pub domain_warp_2_amount_b: f32,
    pub domain_warp_2_scale_b: f32,

    pub misc_f: f32,
    pub misc_i: i32,
}

impl Default for ParamsUniform {
    fn default() -> Self {
        Self {
            dimensions: crate::BUFFER_LEN as u32,

            // circle generator
            radius: 0.3,
            noise_seed: 0,
            noise_freq: 0.3,
            noise_amplitude: 0.8,
            noise_offset: 0.0,
            noise_octaves: 5,
            noise_lacunarity: 2.0,
            power_bias: 1.8,
            flatness: 1.5,
            steepness: 1.3,
            mix: 0.5,
            noise_warp_amount: 0.0,
            noise_warp_scale: 0.0,

            // domain warp 1
            domain_warp_1_amount_a: 0.0,
            domain_warp_1_scale_a: 0.0,
            domain_warp_1_amount_b: 0.0,
            domain_warp_1_scale_b: 0.0,
            
            // cellular automata
            noise_weight: 0.53,
            ca_thresh: 0.24,
            ca_search_radius: 3.8,
            ca_edge_pow: 1.5,
            edge_suppress_mix: 1.0,

            // cave domain warp
            domain_warp_2_amount_a: 0.0,
            domain_warp_2_scale_a: 0.0,
            domain_warp_2_amount_b: 0.0,
            domain_warp_2_scale_b: 0.0,

            misc_f: 0.0,
            misc_i: 0,
        }
    }
}


