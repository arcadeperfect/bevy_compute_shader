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

// Define the size of our buffer - one point per degree in a circle
const BUFFER_LEN: usize = 360;

// Resource wrapper for receiving data in the main world
#[derive(Resource, Deref)]
pub struct MainWorldReceiver(Receiver<Vec<f32>>);

// Resource wrapper for sending data to the render world
#[derive(Resource, Deref)]
pub struct RenderWorldSender(Sender<Vec<f32>>);

// System that receives and processes data from the GPU
// pub fn receive(receiver: Res<MainWorldReceiver>) {
//     if let Ok(data) = receiver.try_recv() {
//         println!("Received circle data: {data:?}");
//     }
// }

// Plugin that sets up GPU computation and data readback
pub struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        // Create a channel for communication between main world and render world
        let (s, r) = crossbeam_channel::unbounded();
        app.insert_resource(MainWorldReceiver(r));

        // Configure the render app with necessary resources and systems
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(RenderWorldSender(s))
            .init_resource::<ComputePipeline>()
            .init_resource::<Buffers>()
            .add_systems(
                Render,
                (
                    // System to prepare GPU bind groups if they don't exist
                    prepare_bind_group
                        .in_set(RenderSet::PrepareBindGroups)
                        .run_if(not(resource_exists::<GpuBufferBindGroup>)),
                    // System to map and read buffer after rendering
                    map_and_read_buffer.after(RenderSet::Render),
                ),
            );

        // Add compute node to render graph
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

// Resource containing both GPU and CPU buffers
#[derive(Resource)]
struct Buffers {
    gpu_buffer: BufferVec<f32>,  // Buffer on GPU for computation
    cpu_buffer: Buffer,          // Buffer on CPU for reading results
}

impl FromWorld for Buffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        // Initialize GPU buffer with zeros
        let mut gpu_buffer = BufferVec::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        for _ in 0..BUFFER_LEN {
            gpu_buffer.push(0.0);
        }
        gpu_buffer.write_buffer(render_device, render_queue);

        // Create CPU buffer for reading back results
        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("readback_buffer"),
            size: (BUFFER_LEN * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            gpu_buffer,
            cpu_buffer,
        }
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
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::single(
            buffers
                .gpu_buffer
                .binding()
                .expect("Buffer should have already been uploaded to the gpu"),
        ),
    );
    commands.insert_resource(GpuBufferBindGroup(bind_group));
}

// Resource containing compute pipeline configuration
#[derive(Resource)]
struct ComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        
        // Create bind group layout for compute shader
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                storage_buffer::<Vec<f32>>(false),
            ),
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

        // Execute compute shader if pipeline is ready
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
            pass.dispatch_workgroups(BUFFER_LEN as u32, 1, 1);
        }

        // Copy results from GPU buffer to CPU buffer
        let buffers = world.resource::<Buffers>();
        render_context.command_encoder().copy_buffer_to_buffer(
            buffers
                .gpu_buffer
                .buffer()
                .expect("Buffer should have already been uploaded to the gpu"),
            0,
            &buffers.cpu_buffer,
            0,
            (BUFFER_LEN * std::mem::size_of::<f32>()) as u64,
        );

        Ok(())
    }
}
