// #import compute::noise
// #import compute::utils

// struct Params {
//     dimensions: u32,

//     // circle generator
//     radius: f32,
//     noise_seed: u32,
//     noise_freq: f32,
//     noise_amplitude: f32,
//     noise_offset: f32,
//     power_bias: f32,
//     flatness: f32,
//     steepness: f32,
//     mix: f32,
//     noise_warp_amount: f32,
//     noise_warp_scale: f32,

//     // domain warp 1
//     domain_warp_1_amount_1: f32,
//     domain_warp_1_scale_1: f32,
//     domain_warp_1_amount_2: f32,
//     domain_warp_1_scale_2: f32,
    
//     // cellular automata
//     noise_weight: f32,
//     ca_thresh: f32,
//     ca_search_radius: f32,
//     ca_edge_pow: f32,
//     edge_suppress_mix: f32,

//     // cave domain warp
//     domain_warp_2_amount_1: f32,
//     domain_warp_2_scale_1: f32,
//     domain_warp_2_amount_2: f32,
//     domain_warp_2_scale_2: f32,
// }

// const BUFFER_LEN = 1024u;
// struct DataGrid{
//     floats: array<array<array<f32, 8>, BUFFER_LEN>, BUFFER_LEN>,
//     ints: array<array<array<i32, 8>, BUFFER_LEN>, BUFFER_LEN>,
// };

// @group(0) @binding(0) var<uniform> params: Params;
// @group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
// @group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
// @group(0) @binding(3) var<storage, read_write> input_grid: DataGrid;
// // @group(0) @binding(4) var<storage, read_write> output_grid: DataGrid;


// fn sample_with_offset(pos: vec2<i32>, offset: vec2<f32>) -> f32 {
//     let dim = f32(params.dimensions);
//     let new_pos = vec2<i32>(
//         i32(clamp(f32(pos.x) + offset.x * dim, 0.0, dim - 1.0)),
//         i32(clamp(f32(pos.y) + offset.y * dim, 0.0, dim - 1.0))
//     );
//     return textureLoad(input_texture, new_pos).x;
// }

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
//     let noise_pos = pos * params.domain_warp_2_scale_1;
//     let noise_x = noise::fbm(noise_pos + vec2<f32>(0.0, 0.0));
//     let noise_y = noise::fbm(noise_pos + vec2<f32>(3.33, 2.77));
    
//     // Create offset vector
//     let offset = vec2f(
//         noise_x * params.domain_warp_2_amount_1,
//         noise_y * params.domain_warp_2_amount_1
//     );
    
//     // Sample the texture with the warped coordinates
//     let warped_value = sample_with_offset(upos, offset);
    
//     // You can blend between the original and warped version if desired
//     let current = textureLoad(input_texture, upos);
        
//     textureStore(output_texture, upos, vec4f(warped_value, current.g, current.b, current.a));
// }




#import compute::noise
#import compute::utils

struct DomainWarpParams {
    scale_1: f32,
    amount_1: f32,
    scale_2: f32,
    amount_2: f32,
    offset_x: f32,
    offset_y: f32,
}


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
struct DataGrid {
    floats: array<array<array<f32, 8>, BUFFER_LEN>, BUFFER_LEN>,
    ints: array<array<array<i32, 8>, BUFFER_LEN>, BUFFER_LEN>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
// @group(0) @binding(3) var<storage, read_write> input_grid: DataGrid;

fn apply_domain_warp(pos: vec2<f32>, warp_params: DomainWarpParams) -> vec2<f32> {
    // First level of warping
    let noise_pos_1 = pos * warp_params.scale_1;
    let noise_x_1 = noise::fbm(noise_pos_1 + vec2<f32>(warp_params.offset_x, 0.0));
    let noise_y_1 = noise::fbm(noise_pos_1 + vec2<f32>(warp_params.offset_x + 3.33, warp_params.offset_y + 2.77));
    let offset_1 = vec2<f32>(
        noise_x_1 * warp_params.amount_1,
        noise_y_1 * warp_params.amount_1
    );
    
    // Second level of warping
    let warped_pos = pos + offset_1;
    let noise_pos_2 = warped_pos * warp_params.scale_2;
    let noise_x_2 = noise::fbm(noise_pos_2 + vec2<f32>(warp_params.offset_x + 1.234, 0.0));
    let noise_y_2 = noise::fbm(noise_pos_2 + vec2<f32>(warp_params.offset_x + 4.56, warp_params.offset_y + 3.89));
    let offset_2 = vec2<f32>(
        noise_x_2 * warp_params.amount_2,
        noise_y_2 * warp_params.amount_2
    );
    
    return offset_1 + offset_2;
}

fn sample_with_offset(pos: vec2<i32>, offset: vec2<f32>) -> f32 {
    let dim = f32(params.dimensions);
    let new_pos = vec2<i32>(
        i32(clamp(f32(pos.x) + offset.x * dim, 0.0, dim - 1.0)),
        i32(clamp(f32(pos.y) + offset.y * dim, 0.0, dim - 1.0))
    );
    return textureLoad(input_texture, new_pos).x;
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
    
    // Set up parameters for second domain warp (cave warp)
    let warp_params = DomainWarpParams(
        params.domain_warp_2_scale_1,
        params.domain_warp_2_amount_1,
        params.domain_warp_2_scale_2,
        params.domain_warp_2_amount_2,
        0.0,  // offset_x
        0.0   // offset_y
    );
    
    // Apply the domain warp
    let offset = apply_domain_warp(pos, warp_params);
    
    // Sample the texture with the warped coordinates
    let warped_value = sample_with_offset(upos, offset);
    
    // Preserve other channels from the current texture
    let current = textureLoad(input_texture, upos);
    
    textureStore(output_texture, upos, vec4<f32>(warped_value, current.g, current.b, current.a));
}