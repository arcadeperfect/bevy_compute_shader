
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
        texture::GpuImage,
    },
    utils::HashMap,
};
use bytemuck::bytes_of;

use crate::{parameters::ParamsUniform, pipeline::ComputePipelines, BindGroupSelection, GpuBufferBindGroups, ImageBufferContainer, ShaderConfigHolder};

pub fn prepare_bind_groups(
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
    
    let grid_buffer_a = buffers.get(&buffer_container.grid_buffer_a).unwrap();
    let grid_buffer_b = buffers.get(&buffer_container.grid_buffer_b).unwrap();
    
    let strip_buffer_a = buffers.get(&buffer_container.strip_buffer_a).unwrap();
    let strip_buffer_b = buffers.get(&buffer_container.strip_buffer_b).unwrap();

    let image_a1 = images.get(&buffer_container.tex_buffer_a1).unwrap();
    let image_b1 = images.get(&buffer_container.tex_buffer_b1).unwrap();
    let image_a2 = images.get(&buffer_container.tex_buffer_a2).unwrap();
    let image_b2 = images.get(&buffer_container.tex_buffer_b2).unwrap();
    let image_a3 = images.get(&buffer_container.tex_buffer_a3).unwrap();
    let image_b3 = images.get(&buffer_container.tex_buffer_b3).unwrap();
    let result_image = images.get(&buffer_container.result).unwrap();
    let gradient_image = images.get(&buffer_container.grad_texture).unwrap();

    let bind_groups = vec![
        // A -> B
        render_device.create_bind_group(
            None,
            &pipeline.compute_layout,
            &BindGroupEntries::sequential((
                uniform_buffer.as_entire_buffer_binding(),
                image_a1.texture_view.into_binding(),
                image_b1.texture_view.into_binding(),
                image_a2.texture_view.into_binding(),
                image_b2.texture_view.into_binding(),
                image_a3.texture_view.into_binding(),
                image_b3.texture_view.into_binding(),
                grid_buffer_a.buffer.as_entire_buffer_binding(),
                grid_buffer_b.buffer.as_entire_buffer_binding(),
                strip_buffer_a.buffer.as_entire_buffer_binding(),
                strip_buffer_b.buffer.as_entire_buffer_binding(),
                gradient_image.texture_view.into_binding(),
            )),
        ),
        // B -> A
        render_device.create_bind_group(
            None,
            &pipeline.compute_layout,
            &BindGroupEntries::sequential((
                uniform_buffer.as_entire_buffer_binding(),
                image_b1.texture_view.into_binding(),
                image_a1.texture_view.into_binding(),
                image_b2.texture_view.into_binding(),
                image_a2.texture_view.into_binding(),
                image_b3.texture_view.into_binding(),
                image_a3.texture_view.into_binding(),
                grid_buffer_a.buffer.as_entire_buffer_binding(),
                grid_buffer_b.buffer.as_entire_buffer_binding(),
                strip_buffer_a.buffer.as_entire_buffer_binding(),
                strip_buffer_b.buffer.as_entire_buffer_binding(),
                gradient_image.texture_view.into_binding(),
            )),
        ),
    ];

    let extract_a = render_device.create_bind_group(
        None,
        &pipeline.extract_layout,
        &BindGroupEntries::sequential((
            uniform_buffer.as_entire_buffer_binding(),
            image_a1.texture_view.into_binding(),
            image_a2.texture_view.into_binding(),
            image_a3.texture_view.into_binding(),
            result_image.texture_view.into_binding(),
            grid_buffer_a.buffer.as_entire_buffer_binding(),
            grid_buffer_b.buffer.as_entire_buffer_binding(),
            strip_buffer_a.buffer.as_entire_buffer_binding(),
            strip_buffer_b.buffer.as_entire_buffer_binding(),
            gradient_image.texture_view.into_binding(),
        )),
    );
    let extract_b = render_device.create_bind_group(
        None,
        &pipeline.extract_layout,
        &BindGroupEntries::sequential((
            uniform_buffer.as_entire_buffer_binding(),
            image_b1.texture_view.into_binding(),
            image_b2.texture_view.into_binding(),
            image_b3.texture_view.into_binding(),
            result_image.texture_view.into_binding(),
            grid_buffer_a.buffer.as_entire_buffer_binding(),
            grid_buffer_b.buffer.as_entire_buffer_binding(),
            strip_buffer_a.buffer.as_entire_buffer_binding(),
            strip_buffer_b.buffer.as_entire_buffer_binding(),
            gradient_image.texture_view.into_binding(),
        )),
    );

    commands.insert_resource(GpuBufferBindGroups {
        bind_groups,
        final_pass_a: extract_a,
        final_pass_b: extract_b,
        uniform_buffer,
        // grad_buffer:gradient_image
        // iteration: 0,
    });
}

pub fn prepare_bind_group_selection(
    mut commands: Commands,
    pipelines: Res<ComputePipelines>,
    shader_configurator: Res<ShaderConfigHolder>,
) {
    let mut selectors = HashMap::new();
    let mut total_iterations = 0;
    let mut node: u32 = 0;

    // println!("{}", shader_configurator.shader_configs.len());

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
