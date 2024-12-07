#import compute::noise
#import compute::utils

struct Params {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
    noise_offset: f32,
    power_bias: f32,
    flatness: f32,
    steepness:f32,
    mix:f32,
    warp_amount: f32,
    warp_scale: f32, 
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

    let current = textureLoad(input_texture, upos);
    // textureStore(output_texture, upos, current);
    
    // // Just output solid red to verify shader is running
    // textureStore(output_texture, upos, vec4<f32>(1.0, 0.0, 0.0, 1.0));

    
    let dim = f32(params.dimensions);
    
    // Convert position to 0-1 range for noise generation
    let pos = vec2f(f32(x) / dim, f32(y) / dim);
    
    // Generate two noise values for x and y offsets
    let noise_pos = pos * params.warp_scale;
    let noise_x = noise::fbm(noise_pos + vec2<f32>(0.0, 0.0));
    let noise_y = noise::fbm(noise_pos + vec2<f32>(3.33, 2.77));
    
    // Create offset vector
    let offset = vec2f(
        noise_x * params.warp_amount,
        noise_y * params.warp_amount
    );
    
    // Sample the texture with the warped coordinates
    let warped_value = sample_with_offset(upos, offset);
    
    // You can blend between the original and warped version if desired
    let original_value = textureLoad(input_texture, upos);
    
    
    textureStore(output_texture, upos, warped_value);
}