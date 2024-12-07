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

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;

// Power bias to make peaks more pronounced
fn power_bias(n: f32, power: f32) -> f32{
    let normalized = (n + 1.) * 0.5;
    return pow(normalized, power) * 2. -1.; 
}

// Plateau function for flat areas with steep cliffs
fn plateau(n: f32, flatness: f32) -> f32{
    let x = n * flatness;
    let exp2x = exp(2.0 * x);
    return (exp2x -1.0) / (exp2x + 1.0);
}

// Exponential distribution for elevation concentration
fn exp_distribution(n: f32, sharpness: f32) -> f32 {
    let normalized = (n +1.) * 0.5;
    return (exp(n * sharpness) - 1.0) / (exp(sharpness) - 1.0) * 2.0 - 1.0;
}

// Combined mountain bias function
fn mountain_bias(n: f32) -> f32 {
    var result = n;
    // make peaks more pronounced
    result = power_bias(n, params.power_bias);

    // add plateaus
    result = plateau(result, params.flatness);

    // // adjust distribution
    result = exp_distribution(result, params.steepness);

    return result;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Early return if we're outside the texture dimensions
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let dim = params.dimensions;
    
    var pos = vec2f(
        f32(x) / f32(dim),
        f32(y) / f32(dim)
    );


    let seed_mult = 10.0;
    let center = vec2<f32>(0.5, 0.5);
    
    let angle = atan2(pos.x - center.x, pos.y - center.y);
    let seed = vec2f(cos(angle), sin(angle)); 
    var n = noise::fbm((seed * seed_mult * params.noise_scale) + params.noise_offset);

    var m = mountain_bias(n) * 0.03;
    n = n * 0.03;
    
    n = n * params.noise_amplitude;
    m = m * params.noise_amplitude;

    var result = mix(n, m, params.mix);

    let r = params.radius * 0.4 + result;
    let dist = distance(pos, center);
    let upos = vec2<i32>(i32(x), i32(y)); 

    let v = select(0.0, 1.0, dist <= r);

    pos = pos - center;
    let mag = length(pos);
    pos = vec2f(mag, 0.0);
    let edge = vec2f(r, 0.0);

    textureStore(output_texture, upos, vec4<f32>(v , dist, distance(pos, edge), 1.));    
} 

