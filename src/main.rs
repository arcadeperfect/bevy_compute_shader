use std::array;

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        self, extract_resource::{self, ExtractResource, ExtractResourcePlugin}, render_asset::{RenderAssetUsages, RenderAssets}, render_graph::{self, RenderGraph, RenderLabel}, render_resource::{binding_types::texture_storage_2d, *}, renderer::{RenderContext, RenderDevice, RenderQueue}, storage::{GpuShaderStorageBuffer, ShaderStorageBuffer}, texture::GpuImage, Render, RenderApp, RenderSet
    },
    utils::HashMap,
};
use binding_types::{storage_buffer, uniform_buffer};
use bytemuck::{bytes_of, Pod, Zeroable};

mod gui;

const GENERATE_CIRCLE_HANDLE: Handle<Shader> = Handle::weak_from_u128(13378847158248049035);
const DOMAIN_WARP_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378847158248049035);
const PRE_CA_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378547158240049035);
const CA_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378547158248049035);
const EXTRACT_HANDLE: Handle<Shader> = Handle::weak_from_u128(33378847158248049035);
const UTIL_NOISE_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(14378847158248049035);
const UTIL_VECTOR_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(25378847158248049035);

// The length of the buffer sent to the gpu
const BUFFER_LEN: usize = 1024;

#[derive(Resource, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ExtractResource, ShaderType)]
#[repr(C)]
struct ParamsUniform {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_freq: f32,
    noise_amplitude: f32,
    noise_offset: f32,
    power_bias: f32,
    flatness: f32,
    steepness: f32,
    mix: f32,
    warp_amount: f32,
    warp_scale: f32,
    noise_weight: f32,
    ca_thresh: f32,
    ca_search_radius: f32,
    ca_edge_pow: f32,
    edge_suppress_mix: f32,
}

impl Default for ParamsUniform {
    fn default() -> Self {
        Self {
            dimensions: BUFFER_LEN as u32,
            radius: 0.3,
            noise_seed: 0,
            noise_freq: 0.3,
            noise_amplitude: 1.55,
            noise_offset: 0.0,
            power_bias: 1.8,
            flatness: 1.5,
            steepness: 1.3,
            mix: 0.5,
            warp_amount: 0.0,
            warp_scale: 0.0,
            noise_weight: 0.53,
            ca_thresh: 0.24,
            ca_search_radius: 3.8,
            ca_edge_pow: 1.5,
            edge_suppress_mix: 1.0,
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
            // ExtractResourcePlugin::<ShaderConfigurator>::default(),
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
    Compute3,
    Compute4,
    Final,
}

struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        let shader_configs = vec![
            ShaderConfig {
                shader_handle: GENERATE_CIRCLE_HANDLE,
                iterations: 1,
            },
            ShaderConfig {
                shader_handle: DOMAIN_WARP_HANDLE,
                iterations: 5,
            },
            ShaderConfig {
                shader_handle: PRE_CA_HANDLE,
                iterations: 1,
            },
            ShaderConfig {
                shader_handle: CA_HANDLE,
                iterations: 16,
            },
        ];

        app.insert_resource(ShaderConfigurator { shader_configs });
        app.add_plugins(ExtractResourcePlugin::<ShaderConfigurator>::default());

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
            GENERATE_CIRCLE_HANDLE,
            "shaders/generate_circle.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            DOMAIN_WARP_HANDLE,
            "shaders/domain_warp.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(app, CA_HANDLE, "shaders/ca.wgsl", Shader::from_wgsl);
        load_internal_asset!(
            app,
            PRE_CA_HANDLE,
            "shaders/pre_ca_noise.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            EXTRACT_HANDLE,
            "shaders/extract.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut App) {
        let shader_configs = app.world().resource::<ShaderConfigurator>().clone();

        let render_app = app.sub_app_mut(RenderApp);

        render_app.insert_resource(shader_configs);

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
        render_graph.add_node(
            ComputeNodeLabel::Compute3,
            ComputeNode {
                pipeline_index: 2,
                is_final: false,
            },
        );
        render_graph.add_node(
            ComputeNodeLabel::Compute4,
            ComputeNode {
                pipeline_index: 3,
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
        render_graph.add_node_edge(ComputeNodeLabel::Compute2, ComputeNodeLabel::Compute3);
        render_graph.add_node_edge(ComputeNodeLabel::Compute3, ComputeNodeLabel::Compute4);
        render_graph.add_node_edge(ComputeNodeLabel::Compute4, ComputeNodeLabel::Final);
    }
}

#[derive(Resource, ExtractResource, Clone)]
struct ImageBufferContainer {
    tex_buffer_a: Handle<Image>,
    tex_buffer_b: Handle<Image>,
    result: Handle<Image>,
    data_buffer_a:  Handle<ShaderStorageBuffer>,
    data_buffer_b:  Handle<ShaderStorageBuffer>,
}

#[derive(Copy, Clone, Pod, Zeroable, Default, ShaderType)]
#[repr(C)]
struct Grid2D {
    // This creates a 10x20 grid
    data: [[f32; 20]; 10],
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    render_device: Res<RenderDevice>,
) {
    commands.spawn((Camera2d::default(),));

    let size = Extent3d {
        width: BUFFER_LEN as u32,
        height: BUFFER_LEN as u32,
        ..default()
    };

    let mut create_texture_image = || {
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

    let texture_buffer_a = create_texture_image();
    let texture_buffer_b = create_texture_image();
    let result = create_texture_image();



    let grid1 = Grid2D {
        data: [[0.0; 20]; 10],
    };

    let mut buffer1 = ShaderStorageBuffer::from(vec![grid1]);
    buffer1.buffer_description.usage |= BufferUsages::COPY_SRC;
    let buffer1_handle = buffers.add(buffer1);
    
    let grid2 = Grid2D {
        data: [[0.0; 20]; 10],
    };

    let mut buffer2 = ShaderStorageBuffer::from(vec![grid2]);
    buffer2.buffer_description.usage |= BufferUsages::COPY_SRC;

    let buffer2_handle = buffers.add(buffer2);


    commands.spawn((
        Sprite {
            image: result.clone(),
            custom_size: Some(Vec2::splat(1000.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3::splat(1.0)),
    ));

    commands.insert_resource(ImageBufferContainer {
        tex_buffer_a: texture_buffer_a,
        tex_buffer_b: texture_buffer_b,
        result,
        data_buffer_a: buffer1_handle,
        data_buffer_b: buffer2_handle
    });
}

#[derive(Resource)]
struct GpuBufferBindGroups {
    bind_groups: Vec<BindGroup>,
    final_pass_a: BindGroup,
    final_pass_b: BindGroup,
    uniform_buffer: Buffer,
}

#[derive(Resource)]
struct BindGroupSelection {
    // node_bind_groups: Vec<Selector>, // Index of bind group to use for each node
    selectors: HashMap<u32, Vec<u32>>,
    final_pass: u32,
}

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<ComputePipelines>,
    render_device: Res<RenderDevice>,
    buffer_container: Res<ImageBufferContainer>,
    images: Res<RenderAssets<GpuImage>>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
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

    let buffer_a = buffers.get(&buffer_container.data_buffer_a).unwrap();
    let buffer_b = buffers.get(&buffer_container.data_buffer_a).unwrap();

    let image_a = images.get(&buffer_container.tex_buffer_a).unwrap();
    let image_b = images.get(&buffer_container.tex_buffer_b).unwrap();
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
                buffer_a.buffer.as_entire_buffer_binding(),
                buffer_b.buffer.as_entire_buffer_binding(),
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
                buffer_b.buffer.as_entire_buffer_binding(),
                buffer_a.buffer.as_entire_buffer_binding(),
            )),
        ),
    ];

    let extract_a = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            uniform_buffer.as_entire_buffer_binding(),
            image_a.texture_view.into_binding(),
            result_image.texture_view.into_binding(),
            buffer_a.buffer.as_entire_buffer_binding(),
        )),
    );
    let extract_b = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            uniform_buffer.as_entire_buffer_binding(),
            image_b.texture_view.into_binding(),
            result_image.texture_view.into_binding(),
            buffer_b.buffer.as_entire_buffer_binding(),
        )),
    );

    commands.insert_resource(GpuBufferBindGroups {
        bind_groups,
        final_pass_a: extract_a,
        final_pass_b: extract_b,
        uniform_buffer,
        // iteration: 0,
    });
}

#[derive(Resource, Clone, ExtractResource)]
struct ShaderConfigurator {
    shader_configs: Vec<ShaderConfig>,
}

#[derive(Resource)]
struct ComputePipelines {
    layout: BindGroupLayout,
    pipeline_configs: Vec<CachedComputePipelineId>,
    final_pass: CachedComputePipelineId,
}

impl FromWorld for ComputePipelines {
    fn from_world(world: &mut World) -> Self {
        let shader_configurator = world.resource::<ShaderConfigurator>();
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<ParamsUniform>(false),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                    storage_buffer::<Grid2D>(false),
                    storage_buffer::<Grid2D>(false),
                ),
            ),
        );

        let pipeline_cache = world.resource::<PipelineCache>();

        let shader_configs = shader_configurator.shader_configs.clone();

        // Create pipeline for each shader with its iteration count
        let mut pipeline_configs = Vec::new();
        for config in shader_configs {
            let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("compute".into()),
                layout: vec![layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: config.shader_handle,
                shader_defs: Vec::new(),
                entry_point: "main".into(),
                zero_initialize_workgroup_memory: false,
            });

            pipeline_configs.push(pipeline_id);
        }

        let final_pass = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("Final pass".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: EXTRACT_HANDLE,
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

fn prepare_bind_group_selection(
    mut commands: Commands,
    pipelines: Res<ComputePipelines>,
    shader_configurator: Res<ShaderConfigurator>,
) {
    let mut selectors = HashMap::new();
    let mut total_iterations = 0;
    let mut node: u32 = 0;

    for _ in &pipelines.pipeline_configs {
        let mut node_selections = Vec::new();

        let i = shader_configurator.shader_configs[node as usize].iterations;
        for _ in 0..i {
            node_selections.push(total_iterations % 2);
            total_iterations += 1;
        }
        selectors.insert(node, node_selections);
        node += 1;
    }

    let final_pass = total_iterations % 2;

    commands.insert_resource(BindGroupSelection {
        selectors,
        final_pass,
    });
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
        let selectors = world.resource::<BindGroupSelection>();
        let shader_configurator = world.resource::<ShaderConfigurator>();

        if self.is_final {
            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipelines.final_pass) {
                encoder.push_debug_group("Final pass");

                {
                    let group = if selectors.final_pass == 0 {
                        &bind_groups.final_pass_a
                    } else {
                        &bind_groups.final_pass_b
                    };

                    let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
                    pass.set_bind_group(0, group, &[]);
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
            let pipeline_id = pipelines.pipeline_configs[self.pipeline_index];

            if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
                let iters = shader_configurator.shader_configs[self.pipeline_index].iterations;

                for iteration in 0..iters {
                    encoder.push_debug_group(&format!(
                        "Compute pass {} iteration {}",
                        self.pipeline_index, iteration
                    ));

                    {
                        let node = self.pipeline_index as u32;
                        let selection = selectors.selectors[&node][iteration as usize];
                        let mut pass =
                            encoder.begin_compute_pass(&ComputePassDescriptor::default());
                        pass.set_bind_group(0, &bind_groups.bind_groups[selection as usize], &[]);
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

