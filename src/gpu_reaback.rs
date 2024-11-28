//gpu_readback.rs
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::storage_buffer, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};
use crossbeam_channel::{Receiver, Sender};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};

// Resource wrapper for receiving data in the main world
#[derive(Resource, Deref)]
pub struct MainWorldReceiver(Receiver<Vec<f32>>);

// Resource wrapper for sending data to the render world
#[derive(Resource, Deref)]
pub struct RenderWorldSender(Sender<Vec<f32>>);

#[derive(Resource, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ExtractResource)]
#[repr(C)]
pub struct CircleUniforms {
    pub size: u32,
    pub radius: f32,
}

#[derive(Resource)]
struct BufferNeedsRecreation(bool);

// Plugin that sets up GPU computation and data readback
pub struct GpuReadbackPlugin {
    size: u32,
    radius: f32,
}

impl GpuReadbackPlugin {
    pub fn new(size: u32, radius: f32) -> Self {
        Self { size, radius }
    }
}

impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CircleUniforms {
            size: self.size,
            radius: self.radius,
        })
        .add_plugins(ExtractResourcePlugin::<CircleUniforms>::default());  // Add this line

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .insert_resource(CircleUniforms {
                    size: self.size,
                    radius: self.radius,
                })
                .insert_resource(BufferNeedsRecreation(false))
                .add_systems(
                    Render,
                    (
                        update_circle_uniforms.in_set(RenderSet::Prepare),
                        recreate_buffers_if_needed.in_set(RenderSet::Prepare),
                    ),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();
        app.insert_resource(MainWorldReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(RenderWorldSender(s))
            .init_resource::<ComputePipeline>()
            .init_resource::<Buffers>()
            .add_systems(
                Render,
                (
                    prepare_bind_group
                        .in_set(RenderSet::PrepareBindGroups)
                        .run_if(|buffers: Res<Buffers>| buffers.is_changed()),
                    map_and_read_buffer.after(RenderSet::Render),
                ),
            );

        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}
fn update_circle_uniforms(
    uniforms: Res<CircleUniforms>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    buffers: Res<Buffers>,
    mut needs_recreation: ResMut<BufferNeedsRecreation>,
) {
    // Check if uniforms have changed (will be true when extracted from main world)
    if uniforms.is_changed() || uniforms.size != buffers.current_size {
        needs_recreation.0 = true;
    }
    
    render_queue.write_buffer(
        &buffers.uniform_buffer,
        0,
        bytemuck::cast_slice(&[*uniforms]),
    );
}

// Resource containing both GPU and CPU buffers
#[derive(Resource)]
pub struct Buffers {
    gpu_buffer: BufferVec<f32>, // Buffer on GPU for computation
    cpu_buffer: Buffer,         // Buffer on CPU for reading results
    pub uniform_buffer: Buffer,
    current_size: u32,
}

impl FromWorld for Buffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();
        let uniforms = world.resource::<CircleUniforms>();
        let buffer_size = (uniforms.size * uniforms.size) as usize;

        let mut gpu_buffer = BufferVec::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        for _ in 0..buffer_size {
            gpu_buffer.push(0.0);
        }
        gpu_buffer.write_buffer(render_device, render_queue);

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (buffer_size * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("circle_uniforms"),
            size: std::mem::size_of::<CircleUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        render_queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));

        Self {
            gpu_buffer,
            cpu_buffer,
            uniform_buffer,
            current_size: uniforms.size,
        }
    }
}

fn recreate_buffers_if_needed(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    uniforms: Res<CircleUniforms>,
    mut buffers: ResMut<Buffers>,
    mut needs_recreation: ResMut<BufferNeedsRecreation>,
    mut commands: Commands,
) {
    if needs_recreation.0 {
        let buffer_size = (uniforms.size * uniforms.size) as usize;

        // Recreate GPU buffer
        let mut new_gpu_buffer = BufferVec::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        new_gpu_buffer.reserve(buffer_size, &render_device);
        for _ in 0..buffer_size {
            new_gpu_buffer.push(0.0);
        }
        new_gpu_buffer.write_buffer(&render_device, &render_queue);

        // Recreate CPU buffer
        let new_cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (buffer_size * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Update buffers
        buffers.gpu_buffer = new_gpu_buffer;
        buffers.cpu_buffer = new_cpu_buffer;
        buffers.current_size = uniforms.size;

        // Remove existing bind group to force recreation
        commands.remove_resource::<GpuBufferBindGroup>();
        
        needs_recreation.0 = false;
    }
}

// Resource wrapper for GPU bind group
#[derive(Resource)]
struct GpuBufferBindGroup(BindGroup);

// System to prepare bind group for GPU computation
fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
) {
    // Remove the -> impl IntoSystemConfigs
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: buffers
                    .gpu_buffer
                    .binding()
                    .expect("Buffer should be on GPU")
                    .clone(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buffers.uniform_buffer.as_entire_binding(),
            },
        ],
    );
    commands.insert_resource(GpuBufferBindGroup(bind_group));
}

// Add Resource derive and make struct fields public
#[derive(Resource)]
struct ComputePipeline {
    pub layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Create bind group layout for compute shader
        let layout = render_device.create_bind_group_layout(
            None,
            &[
                // Storage buffer binding
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Uniform buffer binding
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        );

        // Load and configure compute shader
        let shader = world.load_asset("shaders/gpu_circle.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("Circle generation compute shader".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
        });
        ComputePipeline { layout, pipeline }
    }
}

// System to map GPU buffer to CPU and read data
fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
    sender: Res<RenderWorldSender>,
) {
    let buffer_slice = buffers.cpu_buffer.slice(..);
    let (s, r) = crossbeam_channel::unbounded::<()>();

    // Asynchronously map the buffer for reading
    buffer_slice.map_async(MapMode::Read, move |r| match r {
        Ok(_) => s.send(()).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    // Wait for mapping to complete
    render_device.poll(Maintain::wait()).panic_on_timeout();
    r.recv().expect("Failed to receive the map_async message");

    // Read data from mapped buffer and send to main world
    {
        let buffer_view = buffer_slice.get_mapped_range();
        let data = buffer_view
            .chunks(std::mem::size_of::<f32>())
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("should be a f32")))
            .collect::<Vec<f32>>();
        sender
            .send(data)
            .expect("Failed to send data to main world");
    }

    // Unmap the buffer
    buffers.cpu_buffer.unmap();
}

// Label for compute node in render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

// Node that performs GPU computation
#[derive(Default)]
struct ComputeNode {}

// Update the ComputeNode implementation
impl render_graph::Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();
        let uniforms = world.resource::<CircleUniforms>();
        let buffers = world.resource::<Buffers>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Circle generation compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(init_pipeline);
            pass.dispatch_workgroups(uniforms.size, uniforms.size, 1);
        }

        // Copy data from GPU buffer to CPU buffer for readback
        render_context.command_encoder().copy_buffer_to_buffer(
            &buffers.gpu_buffer.buffer().unwrap(),
            0,
            &buffers.cpu_buffer,
            0,
            ((uniforms.size * uniforms.size) as usize * std::mem::size_of::<f32>()) as u64,
        );

        Ok(())
    }
}
