
use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin, render_asset::RenderAssetUsages, render_graph::{self, RenderGraph, RenderLabel}, render_resource::{BufferUsages, Extent3d, PipelineCache, TextureDimension, TextureFormat, TextureUsages}, renderer::RenderContext, storage::ShaderStorageBuffer, Render, RenderApp, RenderSet
    },
};

use crate::{
    bind_groups::{prepare_bind_group_selection, prepare_bind_groups}, compute_node::ComputeNode, constants::*, gradient_editor::update_gradient_texture, pipeline::ComputePipelines, update_uniform_buffer, BindGroupSelection, DataGrid, GpuBufferBindGroups, ImageBufferContainer, ShaderConfig, ShaderConfigurator
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
enum ComputeNodeLabel {
    Compute1,
    Compute2,
    Compute3,
    Compute4,
    Compute5,
    Final,
}





pub struct ComputeShaderPlugin;

impl Plugin for ComputeShaderPlugin {
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
            ShaderConfig {
                shader_handle: POST_CA_HANDLE,
                iterations: 1,
            },
        ];

        app.insert_resource(ShaderConfigurator { shader_configs });
        app.add_plugins(ExtractResourcePlugin::<ShaderConfigurator>::default());

        load_shaders(app);
    
        app.add_systems(Startup, setup);
    }

    fn finish(&self, app: &mut App) {
        let shader_configs = app.world().resource::<ShaderConfigurator>().clone();

        let render_app = app.sub_app_mut(RenderApp);

        render_app.insert_resource(shader_configs);

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

        add_nodes(&mut render_graph);
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // commands.spawn((Camera2d::default(), CameraController::default()));

    let buffer_size = std::mem::size_of::<f32>() * GRID_SIZE * BUFFER_LEN * BUFFER_LEN
        + std::mem::size_of::<i32>() * GRID_SIZE * BUFFER_LEN * BUFFER_LEN;

    let mut buffer1 =
        ShaderStorageBuffer::new(&vec![0u8; buffer_size], RenderAssetUsages::RENDER_WORLD);
    buffer1.buffer_description.usage |= BufferUsages::COPY_SRC;

    let mut buffer2 =
        ShaderStorageBuffer::new(&vec![0u8; buffer_size], RenderAssetUsages::RENDER_WORLD);
    buffer2.buffer_description.usage |= BufferUsages::COPY_SRC;

    let buffer1_handle = buffers.add(buffer1);
    let buffer2_handle = buffers.add(buffer2);

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
        let texture_usages = TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
        image.texture_descriptor.usage |= texture_usages;
        images.add(image)
    };

    let texture_buffer_a = create_texture_image();
    let texture_buffer_b = create_texture_image();
    let result = create_texture_image();

    let grid1 = DataGrid {
        floats: [[[0.0; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
        ints: [[[0; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
    };

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
        tex_buffer_a: texture_buffer_a,
        tex_buffer_b: texture_buffer_b,
        result,
        data_buffer_a: buffer1_handle,
        data_buffer_b: buffer2_handle,
        grad_texture: grad_texture_handle,
    });
}

fn add_nodes(render_graph: &mut RenderGraph){
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
    render_graph.add_node(
        ComputeNodeLabel::Compute5,
        ComputeNode {
            pipeline_index: 4,
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
    render_graph.add_node_edge(ComputeNodeLabel::Compute4, ComputeNodeLabel::Compute5);
    render_graph.add_node_edge(ComputeNodeLabel::Compute5, ComputeNodeLabel::Final);
}


fn load_shaders(app: &mut App) {
    load_internal_asset!(
        app,
        COMMON_HANDLE,
        "shaders/common.wgsl",
        Shader::from_wgsl
    );
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
    load_internal_asset!(
        app,
        PRE_CA_HANDLE,
        "shaders/pre_ca_noise.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(app, CA_HANDLE, "shaders/ca.wgsl", Shader::from_wgsl);
    load_internal_asset!(
        app,
        POST_CA_HANDLE,
        "shaders/post_ca_warp.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        EXTRACT_HANDLE,
        "shaders/extract.wgsl",
        Shader::from_wgsl
    );

    app.add_systems(Startup, setup);
}