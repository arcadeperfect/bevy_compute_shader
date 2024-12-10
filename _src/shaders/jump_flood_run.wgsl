// // jump flood run (in progress)

// /*

// Use jump flood to generate a distance field to the nearest boundary 

// Requires a previous step to mark the edges

// Requires multiple iterations

// */


// #import compute::noise
// #import compute::utils

// struct Params {
//     dimensions: u32,

//     // circle generator
//     radius: f32,
//     noise_seed: u32,
//     noise_freq: f32,
//     noise_amplitude: f32,
//     nois_lacunarity:f32,
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
// @group(0) @binding(4) var<storage, read_write> output_grid: DataGrid;
// @group(0) @binding(5) var grad_texture: texture_storage_2d<rgba32float, read>;

// // fn is_valid_point(p: vec2<i32>) -> bool {
// //     return p.x >= 0 && 
// //            p.y >= 0 && 
// //            p.x < i32(params.dimensions) && 
// //            p.y < i32(params.dimensions);
// // }

// @compute @workgroup_size(16, 16)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let x = global_id.x;
//     let y = global_id.y;
    
//     if (x >= params.dimensions || y >= params.dimensions) {
//         return;
//     }

//     let upos = vec2<i32>(i32(x), i32(y));

//     // textureStore(output_texture, upos, vec4f(1.0));
    

//     // let steps = input_grid.ints[0][0][0];
//     // if(steps > 2){
//     //     return;
//     // }

//     // var min_distance = 100000000.0;
//     // for (var dy = -1; dy <= 1; dy++) {
//     //     for (var dx = -1; dx <= 1; dx++) {
//     //         let sample_pos = coords + vec2<i32>(dx, dy) * step;
            
//     //         if (is_valid_point(sample_pos)) {
//     //             let sample = textureLoad(input_texture, sample_pos);
                
//     //             // If this is a boundary point (indicated by value 0.0)
//     //             if (sample.x == 0.0) {
//     //                 let offset = vec2<f32>(coords - sample_pos);
//     //                 let dist = length(offset);
//     //                 min_distance = min(min_distance, dist);
//     //             }
//     //         }
//     //     }
//     // }

//     // input_grid[0][0][0] = steps / 2;

//     // textureStore(output_texture, coords, vec4<f32>(min_distance, 1.0, 0.0, 0.0));

// }