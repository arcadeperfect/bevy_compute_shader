//! Simple example demonstrating the use of the [`Readback`] component to read back data from the GPU
//! using both a storage buffer and texture.

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        gpu_readback::{Readback, ReadbackComplete},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
};

mod gui;

use binding_types::uniform_buffer;
use bytemuck::bytes_of;
use gui::ParamsChanged;

/// This example uses a shader source file from the assets subdirectory
const SHADER1_ASSET_PATH: &str = "shaders/generate_circle.wgsl";
const SHADER2_ASSET_PATH: &str = "shaders/second_pass.wgsl";
const NOISE_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(13378847158248049035);
const VECTOR_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378847158248049035);

// The length of the buffer sent to the gpu
const BUFFER_LEN: usize = 1000;

#[derive(Resource, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ExtractResource, ShaderType)]
#[repr(C)]
struct ParamsUniform {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
}

impl Default for ParamsUniform {
    fn default() -> Self {
        Self {
            dimensions: BUFFER_LEN as u32,
            radius: 0.5,
            noise_seed: 0,
            noise_scale: 1.0,
            noise_amplitude: 1.0,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ParamsUniform::default())
        .add_plugins((
            DefaultPlugins,
            GpuReadbackPlugin,
            ExtractResourcePlugin::<ReadbackImage>::default(),
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

// We need a plugin to organize all the systems and render node required for this example
struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        
        // Load the noise shader first as an internal asset
        load_internal_asset!(
            app,
            NOISE_SHADER_HANDLE,
            "../assets/shaders/utils/noise.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            VECTOR_SHADER_HANDLE,
            "../assets/shaders/utils/utils.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputePipelines>()
        .add_systems(
            Render,
            (
                update_uniform_buffer,
                prepare_bind_groups
                    .in_set(RenderSet::PrepareBindGroups)
                    // We don't need to recreate the bind group every frame
                    .run_if(not(resource_exists::<GpuBufferBindGroups>)),
            ),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        
        render_graph.add_node(ComputeNodeLabel1, ComputeNode{ pass_index: 0});
        render_graph.add_node(ComputeNodeLabel2, ComputeNode{ pass_index: 1});
        render_graph.add_node_edge(ComputeNodeLabel1, ComputeNodeLabel2);

        // render_app
        //     .world_mut()
        //     .resource_mut::<RenderGraph>()
        //     .add_node(ComputeNodeLabel, ComputeNode::default());
        render_app.add_event::<ParamsChanged>();
    }
}

#[derive(Resource, ExtractResource, Clone)]
struct ReadbackImage {
    ping: Handle<Image>,
    pong: Handle<Image>,
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Camera2d::default(),));

    // Create a storage texture with some data
    let size = Extent3d {
        width: BUFFER_LEN as u32,
        height: BUFFER_LEN as u32,
        ..default()
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba32Float,
        // TextureFormat::R32Uint,
        RenderAssetUsages::RENDER_WORLD,
    );

    // We also need to enable the COPY_SRC, as well as STORAGE_BINDING so we can use it in the
    // compute shader
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
    let image = images.add(image);

    let mut create_image = || {
        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba32Float,
            // TextureFormat::R32Uint,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
        images.add(image)
    };

    let ping = create_image();
    let pong = create_image();
    commands.spawn(Readback::texture(ping.clone()));

    // Spawn the readback components. For each frame, the data will be read back from the GPU
    // asynchronously and trigger the `ReadbackComplete` event on this entity. Despawn the entity
    // to stop reading back the data.

    // Textures can also be read back from the GPU. Pay careful attention to the format of the
    // texture, as it will affect how the data is interpreted.
    // commands.spawn(Readback::texture(image.clone()));

    commands.spawn((
        // Sprite::from_image(image.clone()),
        Sprite {
            image: ping.clone(),
            custom_size: Some(Vec2::splat(1000.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3::splat(1.0)),
    ));

    // This is just a simple way to pass the image handle to the render app for our compute node
    // commands.insert_resource(ReadbackImage(image));
    commands.insert_resource(ReadbackImage {
        ping: ping.clone(),
        pong: pong,
    });
}

#[derive(Resource)]
struct GpuBufferBindGroups {
    first_pass: BindGroup,
    second_pass: BindGroup,
    uniform_buffer: Buffer,
}
// #[derive(Resource)]
// struct GpuBufferBindGroup {
//     bind_group: BindGroup,
//     uniform_buffer: Buffer,
// }

fn prepare_bind_groups(
    mut commands: Commands,
    pipeline: Res<ComputePipelines>,
    render_device: Res<RenderDevice>,
    image: Res<ReadbackImage>,
    images: Res<RenderAssets<GpuImage>>,
    params: Res<ParamsUniform>,
    render_queue: Res<RenderQueue>,
) {
    let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("uniform"),
        size: std::mem::size_of::<ParamsUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    render_queue.write_buffer(&uniform_buffer, 0, bytes_of(&*params));

    let ping_image = images.get(&image.ping).unwrap();
    let pong_image = images.get(&image.pong).unwrap();

    let first_pass = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            uniform_buffer.as_entire_buffer_binding(),
            ping_image.texture_view.into_binding(),
            pong_image.texture_view.into_binding(),
        )),
    );
    let second_pass = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            uniform_buffer.as_entire_buffer_binding(),
            pong_image.texture_view.into_binding(),
            ping_image.texture_view.into_binding(),
        )),
    );
    commands.insert_resource(GpuBufferBindGroups {
        first_pass,
        second_pass,
        uniform_buffer,
    });
}

#[derive(Resource)]
struct ComputePipelines {
    layout: BindGroupLayout,
    first_pass: CachedComputePipelineId,
    second_pass: CachedComputePipelineId,
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
        let shader1 = world.load_asset(SHADER1_ASSET_PATH);

        let pipeline_cache = world.resource::<PipelineCache>();
        let first_pass = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("First pass".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader1.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });
        let shader2 = world.load_asset(SHADER2_ASSET_PATH);
        let second_pass = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("First pass".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader2.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });

        ComputePipelines {
            layout,
            first_pass,
            second_pass,
        }
    }
}

// #[derive(Resource)]
// struct ComputePipeline {
//     layout: BindGroupLayout,
//     pipeline: CachedComputePipelineId,
// }

// impl FromWorld for ComputePipeline {
//     fn from_world(world: &mut World) -> Self {
//         let render_device = world.resource::<RenderDevice>();
//         let layout = render_device.create_bind_group_layout(
//             None,
//             &BindGroupLayoutEntries::sequential(
//                 ShaderStages::COMPUTE,
//                 (
//                     uniform_buffer::<ParamsUniform>(false),
//                     texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
//                     texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
//                 ),
//             ),
//         );
//         let shader = world.load_asset(SHADER1_ASSET_PATH);

//         let pipeline_cache = world.resource::<PipelineCache>();
//         let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
//             label: Some("GPU readback compute shader".into()),
//             layout: vec![layout.clone()],
//             push_constant_ranges: Vec::new(),
//             shader: shader.clone(),
//             shader_defs: Vec::new(),
//             entry_point: "main".into(),
//             zero_initialize_workgroup_memory: false,
//         });
//         ComputePipeline { layout, pipeline }
//     }
// }

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel1;
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel2;

/// The node that will execute the compute shader
#[derive(Default)]
struct ComputeNode {
    pass_index: u32,
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

        let (pipeline_id, bind_group) = match self.pass_index {
            0 => (pipelines.first_pass, &bind_groups.first_pass),
            1 => (pipelines.second_pass, &bind_groups.second_pass),
            _ => return Ok(()),
        };

        if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, bind_group, &[]);
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(BUFFER_LEN as u32, BUFFER_LEN as u32, 1);
        }

        // if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
        //     let mut pass =
        //         render_context
        //             .command_encoder()
        //             .begin_compute_pass(&ComputePassDescriptor {
        //                 label: Some("GPU readback compute pass"),
        //                 ..default()
        //             });

        //     pass.set_bind_group(0, &bind_group.bind_group, &[]);
        //     pass.set_pipeline(init_pipeline);
        //     pass.dispatch_workgroups(BUFFER_LEN as u32, BUFFER_LEN as u32, 1);
        // }
        Ok(())
    }
}
