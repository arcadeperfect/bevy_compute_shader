#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, DataGrid}


@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read_write> input_grid: DataGrid;
// @group(0) @binding(4) var<storage, read_write> output_grid: DataGrid;

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

    let upos = vec2<i32>(i32(x), i32(y)); 
    
    let current = textureLoad(input_texture, upos);
    // textureStore(output_texture, upos, current);
    
    
    let dim = params.dimensions;
    
    // normalize the coordinates
    var normd_pos = vec2f(
        f32(x) / f32(dim),
        f32(y) / f32(dim)
    );

    let seed_mult = 10.0;
    let center = vec2<f32>(0.5, 0.5);
    
    let angle = atan2(normd_pos.x - center.x, normd_pos.y - center.y);
    let seed = vec2f(cos(angle), sin(angle)); 
    // var n = noise::fbml((seed * seed_mult * params.noise_freq) + params.noise_offset, params.noise_lacunarity);
    // var n = noise::fbm((seed * seed_mult * params.noise_freq) + params.noise_offset);
    var n = noise::fbml((seed * seed_mult * params.noise_freq) + params.noise_offset, params.noise_lacunarity);

    var m = mountain_bias(n) * 0.03;
    n = n * 0.03;
    
    n = n * params.noise_amplitude;
    m = m * params.noise_amplitude;

    var result = mix(n, m, params.mix);

    let deformed_radius = params.radius + result;
    let dist_to_center = distance(normd_pos, center);


    let v = select(0, 1, dist_to_center <= deformed_radius);

    normd_pos = normd_pos - center;                 // transform so that 0,0 is the center
    let mag = length(normd_pos);                    // the distance from this pixel to the center
    normd_pos = vec2f(mag, 0.0);                    // create a new vector of the same length, but on the x axis
    let edge = vec2f(deformed_radius, 0.0);         // create a new vector on the same axis, at the distance to the edge
  
    var dist_to_edge = edge.x - normd_pos.x;
  
    // dist_to_edge = dist_to_edge * params.ca_edge_pow;
    textureStore(output_texture, upos, vec4<f32>(
        dist_to_center, 
        dist_to_edge, 
        deformed_radius, 
        dist_to_edge / deformed_radius
        ));   
    input_grid.ints[upos.x][upos.y][0] = v;

} 
