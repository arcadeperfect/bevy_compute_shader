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
    noise_weight: f32,
    ca_thresh: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Early return if we're outside the texture dimensions
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }
    let pos = vec2f(f32(x), f32(y));
    let upos = vec2<i32>(i32(x), i32(y));
    let v = noise::rand11(f32(x * y));
    let s = select(0.,1.,v <= params.noise_weight);


    var current = textureLoad(input_texture, upos);
    // var result = current.x-s;
    // result = clamp(result, 0., 1.);
    textureStore(output_texture, upos, vec4f(current.x, current.y, current.z, f32(s)));
}