// domain_warp_1

#import compute::noise
#import compute::utils


struct Params {
    dimensions: u32,

    // circle generator
    radius: f32,
    noise_seed: u32,
    noise_freq: f32,
    noise_amplitude: f32,
    nois_lacunarity:f32,
    noise_offset: f32,
    power_bias: f32,
    flatness: f32,
    steepness: f32,
    mix: f32,
    noise_warp_amount: f32,
    noise_warp_scale: f32,

    // domain warp 1
    domain_warp_1_amount_1: f32,
    domain_warp_1_scale_1: f32,
    domain_warp_1_amount_2: f32,
    domain_warp_1_scale_2: f32,
    
    // cellular automata
    noise_weight: f32,
    ca_thresh: f32,
    ca_search_radius: f32,
    ca_edge_pow: f32,
    edge_suppress_mix: f32,

    // cave domain warp
    domain_warp_2_amount_1: f32,
    domain_warp_2_scale_1: f32,
    domain_warp_2_amount_2: f32,
    domain_warp_2_scale_2: f32,
}



const BUFFER_LEN = 1024u;
struct DataGrid{
    floats: array<array<array<f32, 8>, BUFFER_LEN>, BUFFER_LEN>,
    ints: array<array<array<i32, 8>, BUFFER_LEN>, BUFFER_LEN>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read_write> input_grid: DataGrid;
@group(0) @binding(4) var<storage, read_write> output_grid: DataGrid;
@group(0) @binding(5) var grad_texture: texture_storage_2d<rgba32float, read>;

// fn sample_with_offset(pos: vec2<i32>, offset: vec2<f32>) -> vec4<f32> {
//     let dim = f32(params.dimensions);
//     let new_pos = vec2<i32>(
//         i32(clamp(f32(pos.x) + offset.x * dim, 0.0, dim - 1.0)),
//         i32(clamp(f32(pos.y) + offset.y * dim, 0.0, dim - 1.0))
//     );
//     return textureLoad(input_texture, new_pos);
// }


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
    return textureLoad(input_texture, new_pos);
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
        params.domain_warp_1_scale_1,
        params.domain_warp_1_amount_1,
        0.0,
        0.0
    );
    let offset1 = apply_domain_warp(pos, warp1_params);
    
    // Second domain warp, applied to the already warped position
    let warp2_params = DomainWarpParams(
        params.domain_warp_1_scale_2,
        params.domain_warp_1_amount_2,
        1.234, // Different offset for variety
        5.678  // Different offset for variety
    );
    let offset2 = apply_domain_warp(pos + offset1, warp2_params);
    
    // Combine the warps
    let final_offset = offset1 + offset2;
    
    // Sample the texture with the combined warped coordinates
    let warped_value = sample_with_offset(upos, final_offset);
    
    textureStore(output_texture, upos, warped_value);
}



// @compute @workgroup_size(16, 16)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let x = global_id.x;
//     let y = global_id.y;
    
//     // Early return if we're outside the texture dimensions
//     if (x >= params.dimensions || y >= params.dimensions) {
//         return;
//     }

//     let upos = vec2<i32>(i32(x), i32(y));

    
//     let dim = f32(params.dimensions);
    
//     // Convert position to 0-1 range for noise generation
//     let pos = vec2f(f32(x) / dim, f32(y) / dim);
    
//     // Generate two noise values for x and y offsets
//     let noise_pos = pos * params.domain_warp_1_scale_1;
//     let noise_x = noise::fbm(noise_pos + vec2<f32>(0.0, 0.0));
//     let noise_y = noise::fbm(noise_pos + vec2<f32>(3.33, 2.77));
    
//     // Create offset vector
//     let offset = vec2f(
//         noise_x * params.domain_warp_1_amount_1,
//         noise_y * params.domain_warp_1_amount_1
//     );
    
//     // Sample the texture with the warped coordinates
//     let warped_value = sample_with_offset(upos, offset);
    
//     // You can blend between the original and warped version if desired
//     // let current = textureLoad(input_texture, upos);
//     // textureStore(output_texture, upos, current);
    
//     textureStore(output_texture, upos, warped_value);
// }