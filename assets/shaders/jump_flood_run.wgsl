// //jump flood run

#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, DataGrid}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read_write> grid_a: DataGrid;
@group(0) @binding(4) var<storage, read_write> grid_b: DataGrid;
@group(0) @binding(5) var grad_texture: texture_storage_2d<rgba32float, read>;


fn is_valid_point(p: vec2<i32>) -> bool {
    return p.x >= 0 && 
           p.y >= 0 && 
           p.x < i32(params.dimensions) && 
           p.y < i32(params.dimensions);
}

// @compute @workgroup_size(16, 16)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

//     let x = global_id.x;
//     let y = global_id.y;
    
//     if (x >= params.dimensions || y >= params.dimensions) {
//         return;
//     }

//     let upos = vec2<i32>(i32(x), i32(y));

//     // let step = grid_a.ints[x][y][7];
    


//     let current = textureLoad(input_texture, upos);
//     let step = i32(current.a);
//     if(step < 2){
//         return;
//     }
//     let v = current.r;

//     var min_distance = 10000000.;

//     for(var dy = -1; dy <= 1; dy ++){
//         for(var dx = -1; dx <=1; dx++)
//         {
//             let sample_pos = upos + vec2<i32>(dx, dy) * step;
//             if (is_valid_point(sample_pos)) {
            
//                 let sample = textureLoad(input_texture, sample_pos);
//                 if (sample.x < min_distance) {  // Instead of checking for exactly 0.0
//                     let offset = vec2<f32>(upos - sample_pos);
//                     let dist = length(offset);
//                     min_distance = min(min_distance, dist);
//                 }
//             }
//         }
//     }
//     // if(x == 0 && y == 0){
//     //     grid_a.ints[x][y][7] = step / 2;
//     // }
//     textureStore(output_texture, upos, vec4<f32>(min_distance, 0.0, 0.0, f32(step/2)));
// }

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    // textureStore(output_texture, upos, textureLoad(input_texture, upos));

    let current = textureLoad(input_texture, upos);
    let step = i32(current.a);
    if(step < 2){
        return;
    }

    var min_distance = current.r;  // Start with current distance

    for(var dy = -1; dy <= 1; dy ++){
        for(var dx = -1; dx <=1; dx++) {
            let sample_pos = upos + vec2<i32>(dx, dy) * step;
            if (is_valid_point(sample_pos)) {
                let sample = textureLoad(input_texture, sample_pos);
                
                if (sample.x < 1000000.0) {  // If this is a boundary point or has distance info
                    let offset = vec2<f32>(upos - sample_pos);
                    let dist = length(offset);
                    if (sample.x < 1.0) {  // If it's a boundary point
                        min_distance = min(min_distance, dist);
                    } else {  // If it has distance info from previous passes
                        min_distance = min(min_distance, sample.x + dist);
                    }
                }
            }
        }
    }

    textureStore(output_texture, upos, vec4<f32>(min_distance, 0.0, 0.0, f32(step)/1.25));
}