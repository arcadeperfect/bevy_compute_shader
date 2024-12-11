use bevy::{
    prelude::*,
    render::{
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::RenderDevice,
    },
};
use binding_types::{storage_buffer, uniform_buffer};

use crate::{parameters::ParamsUniform, DataGrid, ShaderConfigHolder, EXTRACT_HANDLE};

#[derive(Resource)]
pub struct ComputePipelines {
    pub compute_layout: BindGroupLayout,
    pub extract_layout: BindGroupLayout,
    pub pipeline_configs: Vec<CachedComputePipelineId>,
    pub final_pass: CachedComputePipelineId,
}

impl FromWorld for ComputePipelines {
    fn from_world(world: &mut World) -> Self {
        // let shader: Handle<Shader> = world.load_asset(SHADER_ASSET_PATH);
        let shader_configurator = world.resource::<ShaderConfigHolder>();
        let render_device = world.resource::<RenderDevice>();
        let compute_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<ParamsUniform>(false),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    storage_buffer::<DataGrid>(false),
                    storage_buffer::<DataGrid>(false),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                ),
            ),
        );
        let extract_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<ParamsUniform>(false),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    storage_buffer::<DataGrid>(false),
                    storage_buffer::<DataGrid>(false),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                ),
            ),
        );

        let pipeline_cache = world.resource::<PipelineCache>();

        let shader_configs = shader_configurator.shader_configs.clone();

        // Create pipeline for each shader with its iteration count
        let mut pipeline_configs = Vec::new();
        for config in shader_configs {

            let shader = world.load_asset(config.shader_path);

            let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("compute".into()),
                layout: vec![compute_layout.clone()],
                push_constant_ranges: Vec::new(),
                // shader: config.shader_handle,
                shader: shader,
                shader_defs: Vec::new(),
                entry_point: "main".into(),
                zero_initialize_workgroup_memory: false,
            });

            pipeline_configs.push(pipeline_id);
        }

        let final_pass = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("Final pass".into()),
            layout: vec![extract_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: EXTRACT_HANDLE,
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });

        ComputePipelines {
            compute_layout,
            extract_layout,
            pipeline_configs,
            final_pass,
        }
    }
}