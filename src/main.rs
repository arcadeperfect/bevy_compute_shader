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
use bevy::render::render_resource::TextureFormat;
use bevy::sprite::MaterialMesh2dBundle;

const BUFFER_LEN: usize = 360; // One degree per point
const TEXTURE_SIZE: u32 = 256;

#[derive(Component)]
struct CircleDisplay;

#[derive(Resource, Deref)]
struct MainWorldReceiver(Receiver<Vec<f32>>);

#[derive(Resource, Deref)]
struct RenderWorldSender(Sender<Vec<f32>>);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, GpuReadbackPlugin))
        .add_systems(Update, receive)
        .run();
}

fn receive(receiver: Res<MainWorldReceiver>) {
    if let Ok(data) = receiver.try_recv() {
        println!("Received circle data: {data:?}");
    }
}

struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, _app: &mut App) {}

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
                        .run_if(not(resource_exists::<GpuBufferBindGroup>)),
                    map_and_read_buffer.after(RenderSet::Render),
                ),
            );

        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(ComputeNodeLabel, ComputeNode::default());
    }
}

#[derive(Resource)]
struct Buffers {
    gpu_buffer: BufferVec<f32>,
    cpu_buffer: Buffer,
}

impl FromWorld for Buffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut gpu_buffer = BufferVec::new(BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        for _ in 0..BUFFER_LEN {
            gpu_buffer.push(0.0);
        }
        gpu_buffer.write_buffer(render_device, render_queue);

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

#[derive(Resource)]
struct GpuBufferBindGroup(BindGroup);

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

#[derive(Resource)]
struct ComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                storage_buffer::<Vec<f32>>(false),
            ),
        );
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

fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    buffers: Res<Buffers>,
    sender: Res<RenderWorldSender>,
) {
    let buffer_slice = buffers.cpu_buffer.slice(..);
    let (s, r) = crossbeam_channel::unbounded::<()>();

    buffer_slice.map_async(MapMode::Read, move |r| match r {
        Ok(_) => s.send(()).expect("Failed to send map update"),
        Err(err) => panic!("Failed to map buffer {err}"),
    });

    render_device.poll(Maintain::wait()).panic_on_timeout();
    r.recv().expect("Failed to receive the map_async message");

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

    buffers.cpu_buffer.unmap();
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ComputeNodeLabel;

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
