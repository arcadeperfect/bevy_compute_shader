use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        render_asset::RenderAssetUsages,
        render_graph::{RenderGraph, RenderLabel},
        render_resource::{BufferUsages, Extent3d, TextureDimension, TextureFormat, TextureUsages},
        renderer::RenderQueue,
        storage::ShaderStorageBuffer,
        Render, RenderApp, RenderSet,
    },
};

use crate::{
    bind_groups::{prepare_bind_group_selection, prepare_bind_groups},
    compute_node::ComputeNode,
    constants::*,
    data_structures::ShaderConfig,
    gradient_editor::update_gradient_texture,
    parameters::ParamsUniform,
    pipeline::ComputePipelines,
    GpuBufferBindGroups, ImageBufferContainer, ShaderConfigHolder,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
enum ComputeNodeLabel {
    Compute(usize),
    Final,
}

pub struct ComputeShaderPlugin;

impl Plugin for ComputeShaderPlugin {
    fn build(&self, app: &mut App) {
        let shader_configs = vec![
            ShaderConfig {
                shader_path: "shaders/generate_circle.wgsl",
                iterations: 1,
            },
            // ShaderConfig {
            //     shader_path: "shaders/domain_warp_1.wgsl",
            //     iterations: 5,
            // },
            // ShaderConfig {
            //     shader_path: "shaders/ca_prepare.wgsl",
            //     iterations: 1,
            // },
            // ShaderConfig {
            //     shader_path: "shaders/ca_run.wgsl",
            //     iterations: 16,
            // },
            // ShaderConfig {
            //     shader_path: "shaders/domain_warp_2.wgsl",
            //     iterations: 1,
            // },
            ShaderConfig {
                shader_path: "shaders/solidify.wgsl",
                iterations: 5,
            },
            // ShaderConfig {
            //     shader_path: "shaders/jump_flood_prepare.wgsl",
            //     iterations: 1,
            // },
            // ShaderConfig {
            //     shader_path: "shaders/jump_flood_run.wgsl",
            //     iterations: 30,
            // },
        ];

        app.insert_resource(ShaderConfigHolder { shader_configs });
        app.add_plugins(ExtractResourcePlugin::<ShaderConfigHolder>::default());

        load_common_shaders(app);

        app.add_systems(Startup, setup);
    }

    fn finish(&self, app: &mut App) {
        let shader_configs = app.world().resource::<ShaderConfigHolder>().clone();

        let render_app = app.sub_app_mut(RenderApp);

        render_app.insert_resource(shader_configs.clone());

        render_app.init_resource::<ComputePipelines>().add_systems(
            Render,
            (
                update_gradient_texture,
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

        // Generate nodes dynamically
        let mut node_labels: Vec<ComputeNodeLabel> = Vec::new();

        // Create compute nodes
        for (index, _) in shader_configs.shader_configs.iter().enumerate() {
            let label = ComputeNodeLabel::Compute(index);
            node_labels.push(label.clone());

            render_graph.add_node(
                label,
                ComputeNode {
                    pipeline_index: index,
                    is_final: false,
                },
            );
        }

        // Add final pass node
        let final_label = ComputeNodeLabel::Final;
        node_labels.push(final_label.clone());
        render_graph.add_node(
            final_label,
            ComputeNode {
                pipeline_index: 0,
                is_final: true,
            },
        );

        // Add edges between nodes
        for i in 0..node_labels.len() - 1 {
            render_graph.add_node_edge(node_labels[i].clone(), node_labels[i + 1].clone());
        }
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // Datastrip Buffers
    let strip_buffer_size = std::mem::size_of::<f32>() * STRIP_SIZE * STRIP_COUNT
        + std::mem::size_of::<i32>() * STRIP_SIZE * STRIP_COUNT;

    let mut strip_buffer_1 = ShaderStorageBuffer::new(
        &vec![0u8; strip_buffer_size],
        RenderAssetUsages::RENDER_WORLD,
    );
    strip_buffer_1.buffer_description.usage |= BufferUsages::COPY_SRC;
    
    let mut strip_buffer_2 = ShaderStorageBuffer::new(
        &vec![0u8; strip_buffer_size],
        RenderAssetUsages::RENDER_WORLD,
    );
    strip_buffer_2.buffer_description.usage |= BufferUsages::COPY_SRC;

    let strip_buffer_1_handle = buffers.add(strip_buffer_1);
    let strip_buffer_2_handle = buffers.add(strip_buffer_2);
    
    // DataGrid buffers

    let grid_buffer_size = std::mem::size_of::<f32>() * GRID_SIZE * BUFFER_LEN * BUFFER_LEN
        + std::mem::size_of::<i32>() * GRID_SIZE * BUFFER_LEN * BUFFER_LEN;

    let mut grid_buffer_1 = ShaderStorageBuffer::new(
        &vec![0u8; grid_buffer_size],
        RenderAssetUsages::RENDER_WORLD,
    );
    grid_buffer_1.buffer_description.usage |= BufferUsages::COPY_SRC;

    let mut grid_buffer_2 = ShaderStorageBuffer::new(
        &vec![0u8; grid_buffer_size],
        RenderAssetUsages::RENDER_WORLD,
    );
    grid_buffer_2.buffer_description.usage |= BufferUsages::COPY_SRC;

    let grid_buffer_1_handle = buffers.add(grid_buffer_1);
    let grid_buffer_2_handle = buffers.add(grid_buffer_2);

    // Texture buffers

    let texture_size = Extent3d {
        width: BUFFER_LEN as u32,
        height: BUFFER_LEN as u32,
        ..default()
    };

    let mut create_texture_image = || {
        let mut image = Image::new_fill(
            texture_size,
            TextureDimension::D2,
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        );
        let texture_usages = TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
        image.texture_descriptor.usage |= texture_usages;
        images.add(image)
    };

    let texture_buffer_a1 = create_texture_image();
    let texture_buffer_b1 = create_texture_image();
    let texture_buffer_a2 = create_texture_image();
    let texture_buffer_b2 = create_texture_image();
    let texture_buffer_a3 = create_texture_image();
    let texture_buffer_b3 = create_texture_image();
    let result = create_texture_image();

    // Grad Texture

    let mut grad_texture = Image::new_fill(
        Extent3d {
            width: 256,
            height: 256,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        TextureFormat::Rgba32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    grad_texture.texture_descriptor.usage |=
        TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING;
    let grad_texture_handle = images.add(grad_texture);

    commands.spawn((
        Sprite {
            image: result.clone(),
            custom_size: Some(Vec2::splat(1000.0)),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3::splat(1.0)),
    ));

    commands.insert_resource(ImageBufferContainer {
        tex_buffer_a1: texture_buffer_a1,
        tex_buffer_b1: texture_buffer_b1,
        tex_buffer_a2: texture_buffer_a2,
        tex_buffer_b2: texture_buffer_b2,
        tex_buffer_a3: texture_buffer_a3,
        tex_buffer_b3: texture_buffer_b3,
        result,
        grid_buffer_a: grid_buffer_1_handle,
        grid_buffer_b: grid_buffer_2_handle,
        strip_buffer_a: strip_buffer_1_handle,
        strip_buffer_b: strip_buffer_2_handle,
        grad_texture: grad_texture_handle,
    });
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

fn load_common_shaders(app: &mut App) {
    load_internal_asset!(app, COMMON_HANDLE, "shaders/common.wgsl", Shader::from_wgsl);
    load_internal_asset!(
        app,
        UTIL_NOISE_SHADER_HANDLE,
        "shaders/utils/noise.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        UTILS_SHADER_HANDLE,
        "shaders/utils/utils.wgsl",
        Shader::from_wgsl
    );

    // todo: this should be loaded as an asset like the other compute shaders

    load_internal_asset!(
        app,
        EXTRACT_HANDLE,
        "shaders/extract.wgsl",
        Shader::from_wgsl
    );

    app.add_systems(Startup, setup);
}
