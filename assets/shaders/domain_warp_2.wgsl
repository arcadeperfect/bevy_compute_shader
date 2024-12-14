#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, DataGrid}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var itex_1: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var otex_1: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var itex_2: texture_storage_2d<rgba32float, read>;
@group(0) @binding(4) var otex_2: texture_storage_2d<rgba32float, write>;
@group(0) @binding(5) var itex_3: texture_storage_2d<rgba32float, read>;
@group(0) @binding(6) var otex_3: texture_storage_2d<rgba32float, write>;
@group(0) @binding(7) var<storage, read_write> grid_a: DataGrid;
@group(0) @binding(8) var<storage, read_write> grid_b: DataGrid;
@group(0) @binding(9) var grad_tex: texture_storage_2d<rgba32float, read>;


struct DomainWarpParams {
    scale: f32,
    amount: f32,
    offset_x: f32,
    offset_y: f32,
}

fn apply_domain_warp(pos: vec2<f32>, params: DomainWarpParams) -> vec2<f32> {
    let noise_pos = pos * params.scale;
    let noise_x = noise::fbm(noise_pos + vec2<f32>(params.offset_x, 0.0));
    let noise_y = noise::fbm(noise_pos + vec2<f32>(params.offset_x + 3.33, params.offset_y + 2.77));
    
    return vec2<f32>(
        noise_x * params.amount,
        noise_y * params.amount
    );
}

fn sample_with_offset(pos: vec2<i32>, offset: vec2<f32>) -> vec4<f32> {
    let dim = f32(params.dimensions);
    let new_pos = vec2<i32>(
        i32(clamp(f32(pos.x) + offset.x * dim, 0.0, dim - 1.0)),
        i32(clamp(f32(pos.y) + offset.y * dim, 0.0, dim - 1.0))
    );
    return textureLoad(itex_2, new_pos);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Early return if we're outside the texture dimensions
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));    
    let dim = f32(params.dimensions);
    
    // Convert position to 0-1 range for noise generation
    let pos = vec2<f32>(f32(x) / dim, f32(y) / dim);
    
    // First domain warp
    let warp1_params = DomainWarpParams(
        params.domain_warp_2_scale_a,
        params.domain_warp_2_amount_a,
        0.0,
        0.0
    );
    let offset1 = apply_domain_warp(pos, warp1_params);
    
    // Second domain warp, applied to the already warped position
    let warp2_params = DomainWarpParams(
        params.domain_warp_2_scale_b,
        params.domain_warp_2_amount_b,
        1.234, // Different offset for variety
        5.678  // Different offset for variety
    );
    let offset2 = apply_domain_warp(pos + offset1, warp2_params);
    
    // Combine the warps
    let final_offset = offset1 + offset2;
    
    // Sample the texture with the combined warped coordinates
    let warped_value = sample_with_offset(upos, final_offset);
    
    textureStore(otex_2, upos, warped_value);
    // textureStore(otex_2,upos, textureLoad(itex_2,upos)); // todo test using the storage buffer to avoid constantly swapping textures
}