use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, TextureAspect},
        renderer::RenderQueue,
        texture::GpuImage,
    },
};

use crate::{GpuBufferBindGroups, Gradients, ImageBufferContainer};

pub fn update_gradient_texture(
    gradients: Res<Gradients>,
    textures: Res<ImageBufferContainer>,
    images: Res<RenderAssets<GpuImage>>,
    render_queue: Res<RenderQueue>,
) {
    let g = &gradients.gradient;

    let new_grad = g.linear_eval_bevy(256, true);

    let mut rgba_data: Vec<f32> = Vec::with_capacity(256 * 4);
    for cc in new_grad {
        rgba_data.extend_from_slice(&[
            // cc.to_linear().red,
            // cc.to_linear().green,
            // cc.to_linear().blue,
            cc.to_srgba().red,
            cc.to_srgba().green,
            cc.to_srgba().blue,
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
