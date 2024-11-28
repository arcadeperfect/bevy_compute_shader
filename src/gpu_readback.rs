use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};
use crossbeam_channel::{Receiver, Sender};

// This struct represents the data we'll send to our GPU shader
// It needs special derive macros to make it compatible with GPU memory
// Pod and Zeroable ensure it can be safely copied to GPU memory
// ExtractResource allows it to be automatically synchronized between main and render worlds
#[derive(Resource, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ExtractResource)]
#[repr(C)]
pub struct CircleUniforms {
    pub size: u32,
    pub radius: f32,
}

// Settings that live in the main world (CPU side)
// This mirrors CircleUniforms but doesn't need GPU compatibility
#[derive(Resource)]
pub struct CircleSettings {
    pub size: u32,
    pub radius: f32,
}

// Channel endpoints for sending data between the main world and render world
// These use crossbeam channels for thread-safe communication
#[derive(Resource, Deref)]
pub struct MainWorldReceiver(Receiver<Vec<f32>>);

#[derive(Resource, Deref)]
pub struct RenderWorldSender(Sender<Vec<f32>>);

// Event that gets fired when we want to change the circle size
// This lets different parts of our app react to size changes
#[derive(Event)]
pub struct CircleSizeChanged {
    pub new_size: u32,
    pub new_radius: f32,
}

// Main plugin struct that orchestrates our GPU computation system
pub struct GpuReadbackPlugin {
    size: u32,
    radius: f32,
}

impl GpuReadbackPlugin {
    pub fn new(size: u32, radius: f32) -> Self {
        Self { size, radius }
    }
}

// Plugin implementation - this is where we set up all our systems and resources
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        // Set up resources and systems in the main world
        app.insert_resource(CircleUniforms {
            size: self.size,
            radius: self.radius,
        })
        .add_plugins(ExtractResourcePlugin::<CircleUniforms>::default())
        .add_systems(Update, handle_size_changes);

        // Set up resources and systems in the render world
        // The render world runs on a separate thread and handles GPU interaction
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
                        // These systems run during the prepare phase of rendering
                        update_circle_uniforms.in_set(RenderSet::Prepare),
                        recreate_buffers_if_needed.in_set(RenderSet::Prepare),
                    ),
                );
        }
    }

    // finish() runs after build() and sets up additional resources and systems
    fn finish(&self, app: &mut App) {
        // Create communication channel between main and render worlds
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
                    // System to prepare GPU binding groups when buffers change
                    prepare_bind_group
                        .in_set(RenderSet::PrepareBindGroups)
                        .run_if(|buffers: Res<Buffers>| buffers.is_changed()),
                    // System to read data back from GPU after rendering
                    map_and_read_buffer.after(RenderSet::Render),
                ),
            );

        // Add our compute node to the render graph
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

// Holds all the buffers we need for GPU computation
// - gpu_buffer: Stores data on the GPU that our compute shader writes to
// - cpu_buffer: Used to read data back from the GPU
// - uniform_buffer: Stores our uniform values (size and radius)
#[derive(Resource)]
pub struct Buffers {
    gpu_buffer: BufferVec<f32>,
    cpu_buffer: Buffer,
    pub uniform_buffer: Buffer,
    current_size: u32,
}

// FromWorld lets us create Buffers with complex initialization logic
impl FromWorld for Buffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();
        let uniforms = world.resource::<CircleUniforms>();
        let buffer_size = (uniforms.size * uniforms.size) as usize;

        // Create GPU buffer for compute shader output
        let mut gpu_buffer = BufferVec::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        for _ in 0..buffer_size {
            gpu_buffer.push(0.0);
        }
        gpu_buffer.write_buffer(render_device, render_queue);

        // Create CPU buffer for reading back results
        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (buffer_size * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create uniform buffer for shader parameters
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

// Holds the compute pipeline configuration including shader layout
#[derive(Resource)]
struct ComputePipeline {
    pub layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

// Creates our compute pipeline and configures how the shader accesses buffers
impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Define how our shader will access buffers
        // We need two bindings:
        // 1. A storage buffer for output data
        // 2. A uniform buffer for our parameters
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

        // Load and configure our compute shader
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

// Wrapper for the bind group that connects our buffers to the shader
#[derive(Resource)]
struct GpuBufferBindGroup(BindGroup);

// Flag to indicate when buffers need to be recreated (e.g., when size changes)
#[derive(Resource)]
struct BufferNeedsRecreation(bool);

// Label for our compute node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

// The actual compute node that runs our shader
#[derive(Default)]
struct ComputeNode {}

// Implementation of the compute node - this is where the GPU work happens
impl render_graph::Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        // Get all the resources we need
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputePipeline>();
        let bind_group = world.resource::<GpuBufferBindGroup>();
        let uniforms = world.resource::<CircleUniforms>();
        let buffers = world.resource::<Buffers>();

        // Run our compute shader
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
            // Dispatch one workgroup per pixel in our output
            pass.dispatch_workgroups(uniforms.size, uniforms.size, 1);
        }

        // Copy the results from GPU to CPU buffer for reading
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

// System that recreates buffers when the size changes
fn recreate_buffers_if_needed(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    uniforms: Res<CircleUniforms>,
    mut buffers: ResMut<Buffers>,
    mut needs_recreation: ResMut<BufferNeedsRecreation>,
    mut commands: Commands,
) {
    if needs_recreation.0 {
        // Create new buffers with the new size
        let buffer_size = (uniforms.size * uniforms.size) as usize;

        let mut new_gpu_buffer = BufferVec::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        new_gpu_buffer.reserve(buffer_size, &render_device);
        for _ in 0..buffer_size {
            new_gpu_buffer.push(0.0);
        }
        new_gpu_buffer.write_buffer(&render_device, &render_queue);

        let new_cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (buffer_size * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Update our buffers resource
        buffers.gpu_buffer = new_gpu_buffer;
        buffers.cpu_buffer = new_cpu_buffer;
        buffers.current_size = uniforms.size;

        // Remove old bind group so it gets recreated
        commands.remove_resource::<GpuBufferBindGroup>();

        needs_recreation.0 = false;
    }
}

// System that creates the bind group connecting our buffers to the shader
fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
) {
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &[
            // Bind our storage buffer
            BindGroupEntry {
                binding: 0,
                resource: buffers
                    .gpu_buffer
                    .binding()
                    .expect("Buffer should be on GPU")
                    .clone(),
            },
            // Bind our uniform buffer
            BindGroupEntry {
                binding: 1,
                resource: buffers.uniform_buffer.as_entire_binding(),
            },
        ],
    );
    commands.insert_resource(GpuBufferBindGroup(bind_group));
}

// System that updates uniform values and checks if buffers need recreation
fn update_circle_uniforms(
    uniforms: Res<CircleUniforms>,
    render_queue: Res<RenderQueue>,
    buffers: Res<Buffers>,
    mut needs_recreation: ResMut<BufferNeedsRecreation>,
) {
    // Check if we need to recreate buffers
    if uniforms.is_changed() || uniforms.size != buffers.current_size {
        needs_recreation.0 = true;
    }

    // Update uniform buffer with new values
    render_queue.write_buffer(
        &buffers.uniform_buffer,
        0,
        bytemuck::cast_slice(&[*uniforms]),
    );
}

// System that handles size change events
fn handle_size_changes(
    mut events: EventReader<CircleSizeChanged>,
    mut settings: ResMut<CircleSettings>,
    mut uniforms: ResMut<CircleUniforms>,
) {
    for event in events.read() {
        settings.size = event.new_size;
        settings.radius = event.new_radius;
        uniforms.size = event.new_size;
        uniforms.radius = event.new_radius;
    }
}

// System that reads data back from the GPU to CPU memory
// This is a complex operation because we need to carefully synchronize GPU and CPU memory access
fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
    sender: Res<RenderWorldSender>,
) {
    // Get a slice of the buffer we want to read
    let buffer_slice = buffers.cpu_buffer.slice(..);

    // Create a channel for synchronizing the async mapping operation
    // This is needed because GPU operations are asynchronous but we need to wait for the mapping to complete
    let (s, r) = crossbeam_channel::unbounded::<()>();

    // Request to map the buffer into CPU memory
    // This is async because it needs to wait for any GPU operations to complete
    buffer_slice.map_async(MapMode::Read, move |result| match result {
        Ok(_) => s.send(()).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    // Wait for the GPU to finish and the buffer to be mapped
    // poll(Maintain::wait()) blocks until the device is idle
    render_device.poll(Maintain::wait()).panic_on_timeout();
    r.recv().expect("Failed to receive the map_async message");

    // Use a new scope to ensure the mapped memory is released properly
    {
        // Get access to the mapped memory
        let buffer_view = buffer_slice.get_mapped_range();
        
        // Convert the raw bytes into f32 values
        // We process the data in chunks of f32 size (4 bytes)
        let data = buffer_view
            .chunks(std::mem::size_of::<f32>())
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("should be a f32")))
            .collect::<Vec<f32>>();

        // Send the processed data back to the main world
        sender
            .send(data)
            .expect("Failed to send data to main world");
    }

    // Unmap the buffer when we're done
    // This releases the CPU mapping and makes the buffer available for GPU use again
    buffers.cpu_buffer.unmap();
}