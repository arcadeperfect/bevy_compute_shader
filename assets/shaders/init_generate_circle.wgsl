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

    let amp = params.noise_amplitude;
    let radius = params.radius;
    


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
    


    // // graph the noise for debug

    // let index = i32(pos.x * f32(STRIP_SIZE));
    // var v1 =  strip_a.floats[0][index];
    // var v2 =  strip_a.floats[1][index];
    // var v3 =  strip_a.floats[2][index];
    // var v4 =  strip_b.floats[0][index];
    // var nze = vec4f(v1,v2,v3,v4);

    // nze = nze * 0.5 + 0.5;
    // nze = nze * amp;
    // nze = 1-nze;
    
    // let solid1 = select(0.,1., pos.y > nze.x);
    // let solid2 = select(0.,1., pos.y > nze.y);
    // let solid3 = select(0.,1., pos.y > nze.z);
    // let solid4 = select(0.,1., pos.y > nze.w);



    // // textureStore(otex_1,upos, vec4f(solid4, solid2, solid3, 1.));
    
    let centered = pos - 0.5;
    let angle = atan2(centered.y, centered.x);
    
    // // Convert to 0 to 2π range
    let angle_positive = angle + 3.14159265359;
    
    // Map angle to buffer index (0 to buffer_length-1)
    let index = i32(angle_positive / (2.0 * 3.14159265359) * f32(STRIP_SIZE));
    
    // Clamp index to valid range
    // let clamped_index = clamp(index, 0, STRIP_SIZE - 1);

    let nze = strip_b.floats[0][index];
    
    
        
    // Distance from center
    let dist = length(centered);
    
    // Deform the radius using noise
    let deformed_radius = radius + (nze * 0.02 * amp);
    
    // Return 1 inside circle, 0 outside (or you could return smooth falloff)
    let solid = select(0., 1., dist < deformed_radius);
    
    textureStore(otex_1,upos, vec4f(solid, 0., 0., 1.));
} 
