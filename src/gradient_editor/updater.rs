use bevy::{prelude::*, render::{render_asset::RenderAssets, render_resource::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect}, renderer::RenderQueue, texture::GpuImage}};

use crate::{GpuBufferBindGroups, Gradients, ImageBufferContainer};

pub fn update_gradient_texture(
    mut gradients: ResMut<Gradients>,
    textures: Res<ImageBufferContainer>,
    images: Res<RenderAssets<GpuImage>>,
    bind_groups: Option<Res<GpuBufferBindGroups>>,
    render_queue: Res<RenderQueue>,
) {
    println!("botty");
    let g = &gradients.gradient;
    let new_grad = g.linear_eval(256, true);

    let mut rgba_data: Vec<f32> = Vec::with_capacity(256 * 4);
    for color in new_grad {
        rgba_data.extend_from_slice(&[
            color.r() as f32 / 255.,
            color.g() as f32 / 255.,
            color.b() as f32 / 255.,
            // 1.0,
            // 0.0,
            // 1.0,
            1.0,
        ]);
    }

    if let Some(grad_texture) = images.get(&textures.grad_texture) {
        // println!("accessed grad texture");
        
        // Create the full texture data (256x256)
        let mut full_texture_data = Vec::with_capacity(256 * 256 * 4);
        for _ in 0..256 {
            full_texture_data.extend_from_slice(&rgba_data);
        }

        render_queue.write_texture(
            ImageCopyTexture {
                texture: &grad_texture.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            bytemuck::cast_slice(&full_texture_data),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4 * 4), // width * components * size_of::<f32>()
                rows_per_image: Some(256),
            },
            Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
        );
    }
}