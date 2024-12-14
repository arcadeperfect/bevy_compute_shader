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

// /*
// Generate the initial noise which is the starting point for the cellular automata,
// and arrange the data in to the primary texture buffer
// */

// @compute @workgroup_size(16, 16)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let x = global_id.x;
//     let y = global_id.y;
    
//     // let pos = vec2f(f32(x), f32(y));

//     // Early return if we're outside the texture dimensions
//     if (x >= params.dimensions || y >= params.dimensions) {
//         return;
//     }

//     let upos = vec2<i32>(i32(x), i32(y));
    
//     // load the normalized distance to edge
//     let normalized_distance_to_edge = textureLoad(itex_2, upos).a;
    
//     // generate weighted noise for the initial state
//     let nze = noise::rand11(f32(x * (y*y)) + params.misc_f * 100);
//     let weighted_noise = select(0.,1.,nze <= params.noise_weight);
//     let weighted_noise_as_float = f32(weighted_noise);
    
//     // store the noise and the normalized distance to edge in otex 1 for cellular automata
//     textureStore(otex_1, upos, vec4f(weighted_noise_as_float,
//                                     0.0, 
//                                     0.0, 
//                                     1.0));
//     // textureStore(otex_2,upos, textureLoad(itex_2,upos)); // tozeddo test using the storage buffer to avoid constantly swapping textures


//     // textureStore(otex_1, upos, vec4f(weighted_noise_as_float,
//     //                                 0.0, 
//     //                                 0.0, 
//     //                                 1.0));
// }




@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let pos = vec2f(f32(x), f32(y));
    let upos = vec2<i32>(i32(x), i32(y));
    let v = noise::rand11(f32(x * y * y));
    let s = select(0.,1.,v <= params.noise_weight);
    var current = textureLoad(itex_1, upos);

    textureStore(otex_2, upos, vec4f(f32(s), 0., 0., 1.));
    // textureStore(otex_2, upos, vec4f(1.0, 0.0, 1.0, 1.0));
    // textureStore(otex_1, upos, vec4f(0., 0., 0., 0.));
}
