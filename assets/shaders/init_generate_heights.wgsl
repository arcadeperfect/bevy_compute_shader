#import compute::noise
#import compute::utils
#import compute::anoise::{psrdnoise2, fbma, terrain_gpt, terrain_claude, terrain_corrected, generate_varied_terrain}
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

fn linearToCircle(index: f32, total_steps: f32) -> vec2<f32> {

    var t = index / total_steps;
    var a = t * 2.0 * 3.14159265359;
    
    return vec2<f32>(cos(a), sin(a));
}

@compute @workgroup_size(256,1,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let oct = params.noise_octaves;
    let lanc = params.noise_lacunarity;
    let rot = params.misc_f;

    let flat = params.flatness * 4.;
    let steep = params.steepness *4.;
    let mix = params.mix * 4.;

    let x = global_id.x;
    let fx = f32(x);
    if (x >= STRIP_SIZE) {  // or however you pass the length
        return;
    }
    
    

    let coord = linearToCircle(fx, f32(STRIP_SIZE));

    let npos = coord * params.noise_freq * 0.1;
    let base_period = 10000.0; // Example period



    // // let nze1 = fbma(npos, u32(oct), lanc, 0.5, base_period, rot);
    // // let nze2 = terrain_claude(npos, u32(oct), lanc, 0.5, base_period, rot);
    // // let nze3 = terrain_gpt(npos, u32(oct), lanc, 0.5, base_period, rot);
    // let nze4 = terrain_corrected(npos, u32(oct), lanc, 0.5, base_period, rot);




    // // strip_a.floats[0][x] = nze1.x;
    // // strip_a.floats[1][x] = nze2;
    // // strip_a.floats[2][x] = nze3;
    // strip_b.floats[0][x] = nze4;

    let base_settings = vec4<f32>(lanc, 0.5, 10000., 0.0); // lacunarity, gain, period, rot
    let variation_settings = vec3<f32>(flat, steep, mix);   // ridge, warp, erosion
    let terrain = generate_varied_terrain(npos, 8u, base_settings, variation_settings) * 5.;
    
    strip_b.floats[0][x] = terrain;
}