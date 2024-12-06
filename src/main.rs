//! Simple example demonstrating the use of the [`Readback`] component to read back data from the GPU
//! using both a storage buffer and texture.

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
};
use binding_types::uniform_buffer;
use bytemuck::bytes_of;

mod gui;

const SHADER1_HANDLE: Handle<Shader> = Handle::weak_from_u128(13378847158248049035);
const SHADER2_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378847158248049035);
const SHADER3_HANDLE: Handle<Shader> = Handle::weak_from_u128(33378847158248049035);
const UTIL_NOISE_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(14378847158248049035);
const UTIL_VECTOR_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(25378847158248049035);

// The length of the buffer sent to the gpu
const BUFFER_LEN: usize = 1024;

#[derive(Resource)]
struct GlobalIterationCounter(u32);

impl Default for GlobalIterationCounter {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Resource, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ExtractResource, ShaderType)]
#[repr(C)]
struct ParamsUniform {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
    noise_offset: f32,
    warp_amount: f32,
    warp_scale: f32,
}

impl Default for ParamsUniform {
    fn default() -> Self {
        Self {
            dimensions: BUFFER_LEN as u32,
            radius: 0.5,
            noise_seed: 0,
            noise_scale: 1.0,
            noise_amplitude: 1.0,
            noise_offset: 0.0,
            warp_amount: 0.0,
            warp_scale: 0.0,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ParamsUniform::default())
        .add_plugins((
            DefaultPlugins,
            GpuReadbackPlugin,
            ExtractResourcePlugin::<ImageBufferContainer>::default(),
            ExtractResourcePlugin::<ParamsUniform>::default(),
            gui::GuiPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

fn update_uniform_buffer(
    bind_groups: Option<Res<GpuBufferBindGroups>>,
    render_queue: Res<RenderQueue>,
    params: Res<ParamsUniform>,
) {
    if let Some(bind_group) = bind_groups {
        render_queue.write_buffer(&bind_group.uniform_buffer, 0, bytemuck::bytes_of(&*params));
    }
}

#[derive(Debug, Clone)]
struct ShaderConfig {
    shader_handle: Handle<Shader>,
    iterations: u32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
enum ComputeNodeLabel {
    Compute1,
    Compute2,
    // Compute3,
    Final,
}

struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            UTIL_NOISE_SHADER_HANDLE,
            "shaders/utils/noise.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            UTIL_VECTOR_SHADER_HANDLE,
            "shaders/utils/utils.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADER1_HANDLE,
            "shaders/generate_circle.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SHADER2_HANDLE,
            "shaders/domain_warp.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SHADER3_HANDLE,
            "shaders/3rd_pass.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<ComputePipelines>().add_systems(
            Render,
            (
                update_uniform_buffer,
                prepare_bind_groups
                    .in_set(RenderSet::PrepareBindGroups)
                    .run_if(not(resource_exists::<GpuBufferBindGroups>)),
                prepare_bind_group_selection
                    .in_set(RenderSet::PrepareBindGroups)
                    .after(prepare_bind_groups),
            ),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        // Add compute nodes
        render_graph.add_node(
            ComputeNodeLabel::Compute1,
            ComputeNode {
                pipeline_index: 0,
                is_final: false,
            },
        );
        render_graph.add_node(
            ComputeNodeLabel::Compute2,
            ComputeNode {
                pipeline_index: 1,
                is_final: false,
            },
        );

        // Add final pass
        render_graph.add_node(
            ComputeNodeLabel::Final,
            ComputeNode {
                pipeline_index: 0,
                is_final: true,
            },
        );

        // Add edges between nodes
        render_graph.add_node_edge(ComputeNodeLabel::Compute1, ComputeNodeLabel::Compute2);
        render_graph.add_node_edge(ComputeNodeLabel::Compute2, ComputeNodeLabel::Final);
    }
}

#[derive(Resource, ExtractResource, Clone)]
struct ImageBufferContainer {
    buffer_a: Handle<Image>,
    buffer_b: Handle<Image>,
    result: Handle<Image>,
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Camera2d::default(),));

    let size = Extent3d {
        width: BUFFER_LEN as u32,
        height: BUFFER_LEN as u32,
        ..default()
    };

    let mut create_image = || {
        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
        images.add(image)
    };

    let buffer_a = create_image();
    let buffer_b = create_image();
    let result = create_image();

    commands.spawn((
        Sprite {
            image: buffer_a.clone(),
            custom_size: Some(Vec2::splat(1000.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3::splat(1.0)),
    ));

    commands.insert_resource(ImageBufferContainer {
        buffer_a,
        buffer_b,
        result,
        // current_read: BufferIndex::A,
        // iteration: 0,
    });
}

#[derive(Resource)]
struct GpuBufferBindGroups {
    bind_groups: Vec<BindGroup>,
    final_pass: BindGroup,
    uniform_buffer: Buffer,
    // iteration: u32,
}

#[derive(Resource)]
struct BindGroupSelection {
    node_bind_groups: Vec<u32>, // Index of bind group to use for each node
}

fn prepare_bind_group_selection(mut commands: Commands, pipelines: Res<ComputePipelines>) {
    let mut total_iterations = 0;
    let node_bind_groups = pipelines
        .pipeline_configs
        .iter()
        .map(|(_, iterations)| {
            let bind_group = (total_iterations / iterations) % 2;
            total_iterations += iterations;
            bind_group
        })
        .collect();

    commands.insert_resource(BindGroupSelection { node_bind_groups });
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<ComputePipelines>,
    render_device: Res<RenderDevice>,
    buffer_container: Res<ImageBufferContainer>,
    images: Res<RenderAssets<GpuImage>>,
    params_res: Res<ParamsUniform>,
    render_queue: Res<RenderQueue>,
) {
    let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("uniform"),
        size: std::mem::size_of::<ParamsUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    render_queue.write_buffer(&uniform_buffer, 0, bytes_of(&*params_res));

    let image_a = images.get(&buffer_container.buffer_a).unwrap();
    let image_b = images.get(&buffer_container.buffer_b).unwrap();
    let result_image = images.get(&buffer_container.result).unwrap();

    let bind_groups = vec![
        // A -> B
        render_device.create_bind_group(
            None,
            &pipeline.layout,
            &BindGroupEntries::sequential((
                uniform_buffer.as_entire_buffer_binding(),
                image_a.texture_view.into_binding(),
                image_b.texture_view.into_binding(),
            )),
        ),
        // B -> A
        render_device.create_bind_group(
            None,
            &pipeline.layout,
            &BindGroupEntries::sequential((
                uniform_buffer.as_entire_buffer_binding(),
                image_b.texture_view.into_binding(),
                image_a.texture_view.into_binding(),
            )),
        ),
    ];

    let final_pass = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            uniform_buffer.as_entire_buffer_binding(),
            image_a.texture_view.into_binding(),
            result_image.texture_view.into_binding(),
        )),
    );

    commands.insert_resource(GpuBufferBindGroups {
        bind_groups,
        final_pass,
        uniform_buffer,
        // iteration: 0,
    });
}

#[derive(Resource)]
struct ComputePipelines {
    layout: BindGroupLayout,
    pipeline_configs: Vec<(CachedComputePipelineId, u32)>, // (pipeline_id, iterations)
    final_pass: CachedComputePipelineId,
}

impl FromWorld for ComputePipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<ParamsUniform>(false),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let pipeline_cache = world.resource::<PipelineCache>();

        // Define shader configurations
        let shader_configs = vec![
            ShaderConfig {
                shader_handle: SHADER1_HANDLE,
                iterations: 1,
            },
            ShaderConfig {
                shader_handle: SHADER2_HANDLE,
                iterations: 1,
            },
            // ShaderConfig {
            //     shader_handle: SHADER3_HANDLE,
            //     iterations: 1,
            // },
        ];

        // Create pipeline for each shader with its iteration count
        let pipeline_configs = shader_configs
            .into_iter()
            .map(|config| {
                let pipeline_id =
                    pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                        label: Some("compute".into()),
                        layout: vec![layout.clone()],
                        push_constant_ranges: Vec::new(),
                        shader: config.shader_handle,
                        shader_defs: Vec::new(),
                        entry_point: "main".into(),
                        zero_initialize_workgroup_memory: false,
                    });
                (pipeline_id, config.iterations)
            })
            .collect();

        let final_pass = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("Final pass".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: SHADER3_HANDLE,
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });

        ComputePipelines {
            layout,
            pipeline_configs,
            final_pass,
        }
    }
}

#[derive(Default)]
struct ComputeNode {
    pipeline_index: usize,
    is_final: bool,
}

impl render_graph::Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<ComputePipelines>();
        let bind_groups = world.resource::<GpuBufferBindGroups>();
        let encoder = render_context.command_encoder();
        let bind_group_selection = world.resource::<BindGroupSelection>();
        let mut i = 0;

        // println!("Running compute node: {}", if self.is_final { "final" } else {
        //     if self.pipeline_index == 0 { "first" }
        //     else if self.pipeline_index == 1 { "second" }
        //     else { "unknown" }
        // });

        if self.is_final {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipelines.final_pass) {
                encoder.push_debug_group("Final pass");
                {
                    let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
                    pass.set_bind_group(0, &bind_groups.final_pass, &[]);
                    pass.set_pipeline(pipeline);
                    pass.dispatch_workgroups(
                        ((BUFFER_LEN + 15) / 16) as u32,
                        ((BUFFER_LEN + 15) / 16) as u32,
                        1,
                    );
                }
                encoder.pop_debug_group();
            }
        } else {
            let (pipeline_id, iterations) = pipelines.pipeline_configs[self.pipeline_index];

            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
                // let mut encoder = render_context.command_encoder();

                for iteration in 0..iterations {
                    encoder.push_debug_group(&format!(
                        "Compute pass {} iteration {}",
                        self.pipeline_index, iteration
                    ));

                    {
                        let bind_group_idx =
                            bind_group_selection.node_bind_groups[self.pipeline_index];
                        let mut pass =
                            encoder.begin_compute_pass(&ComputePassDescriptor::default());
                        pass.set_bind_group(
                            0,
                            &bind_groups.bind_groups[bind_group_idx as usize],
                            &[],
                        );
                        pass.set_pipeline(pipeline);
                        pass.dispatch_workgroups(
                            ((BUFFER_LEN + 15) / 16) as u32,
                            ((BUFFER_LEN + 15) / 16) as u32,
                            1,
                        );
                    }
                    encoder.pop_debug_group();
                }
            }
        }

        Ok(())
    }
}
