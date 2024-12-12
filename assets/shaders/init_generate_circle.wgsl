#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, STRIP_SIZE, DataGrid, DataStrip}


@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var itex_1: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var otex_1: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var itex_2: texture_storage_2d<rgba32float, read>;
@group(0) @binding(4) var otex_2: texture_storage_2d<rgba32float, write>;
@group(0) @binding(5) var itex_3: texture_storage_2d<rgba32float, read>;
@group(0) @binding(6) var otex_3: texture_storage_2d<rgba32float, write>;
@group(0) @binding(7) var<storage, read_write> grid_a: DataGrid;
@group(0) @binding(8) var<storage, read_write> grid_b: DataGrid;
@group(0) @binding(9) var<storage, read_write> strip_a: DataStrip;
@group(0) @binding(10) var<storage, read_write> strip_b: DataStrip;
@group(0) @binding(11) var grad_tex: texture_storage_2d<rgba32float, read>;

/*
Generate a circle with noise deformed edges, and calculate distance fields

This is the basis of the planet
*/

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
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y)); 
    
    
    
    // let current = textureLoad(itex_1, upos);
    // // textureStore(output_texture, upos, current);
    
    
    let dim = params.dimensions;
    
    // normalize the coordinates
    var pos = vec2f(
        f32(x) / f32(dim),
        f32(y) / f32(dim)
    );
    

    let index = i32(pos.x * f32(STRIP_SIZE));
    var v =  strip_a.floats[0][index];
    v = v * 0.5 + 0.5;

    let solid = select(1.,0., pos.y < v);

    let dx = strip_a.floats[1][index];  // x derivative
    let dy = strip_a.floats[2][index];  // x derivative
    
    let steepness = sqrt(dx * dx + dy * dy) * 0.1;

    let c = steepness * solid;

    textureStore(otex_1,upos, vec4f(c, 0., 0., 1.));

    // steepness

    // // center coordinates
    // pos = pos * 2.0 - 1.0;
    // let angle_radians = atan2(pos.y, pos.x);
    // var angle_normd = angle_radians / 3.1415926535897932384626433832795;
    // angle_normd = angle_normd * 0.5 + 0.5;
    // let angle_mapped = angle_normd * f32(STRIP_SIZE-1);
    // let index = i32(angle_mapped);    
    // var nze = strip_a.floats[0][index];
    // nze = nze * 0.1 * params.radius * params.noise_amplitude;

    // let dist_to_center = length(pos);

    // let rock = select(0., 1., dist_to_center < params.radius + nze);
    // let v = rock;
    
    // let dx = strip_a.floats[1][index];  // x derivative
    // let dy = strip_a.floats[2][index];  // x derivative
    
    // let steepness = sqrt(dx * dx + dy * dy) * 0.1;


    // let c = vec4f(steepness * rock, steepness * rock, 0., 1.);
    // textureStore(otex_1, upos, c);
    
    
    // old

    // let seed = vec2f(cos(angle), sin(angle)); 
    // // var n = noise::fbml((seed * seed_mult * params.noise_freq) + params.noise_offset, params.noise_lacunarity);
    // // var n = noise::fbm((seed * seed_mult * params.noise_freq) + params.noise_offset);
    // var n = noise::fbml((seed * seed_mult * params.noise_freq) + params.noise_offset, params.noise_lacunarity);

    // var m = mountain_bias(n) * 0.03;
    // n = n * 0.03;
    
    // n = n * params.noise_amplitude;
    // m = m * params.noise_amplitude;

    // var result = mix(n, m, params.mix);

    // let deformed_radius = params.radius + result;
    // let dist_to_center = distance(normd_pos, center);
    
    // normd_pos = normd_pos - center;                 // transform so that 0,0 is the center
    // let mag = length(normd_pos);                    // the distance from this pixel to the center
    // normd_pos = vec2f(mag, 0.0);                    // create a new vector of the same length, but on the x axis
    // let edge = vec2f(deformed_radius, 0.0);         // create a new vector on the same axis, at the distance to the edge
    // var dist_to_edge = edge.x - normd_pos.x;
        
    // // Only store the distance fields and deformed radius because the edge will be found bt comparing distance
    // // to deformed radius after domain warping
    // // this saves the number of things we have to warp
    
    // textureStore(otex_2, upos, vec4<f32>(
    //     dist_to_center, 
    //     dist_to_edge, 
    //     deformed_radius, 
    //     dist_to_edge / deformed_radius
    //     ));   
} 
