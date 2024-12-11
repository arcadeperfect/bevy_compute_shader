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


fn test_neighbors(x: i32, y: i32, thin: bool, loaded_value: f32) -> f32 {

// comppare this texel to its neighbors
    // if any of the neighbors are different, return 1

    var found = 0.0;
    let dim = i32(params.dimensions);

    // in thin mode, only consifer half the scenarios to prevent double counting
    if(thin && loaded_value == 0.){
        return 0.;
    }

    for(var i = -1; i < 2; i++) {
        for(var j = -1; j <= 2; j++) {
            
            if(i == 0 && j == 0) { // skip self
                continue;
            }

            let new_x = x + i;
            let new_y = y + j;
            let offset = 0;
            if(new_x < offset || new_x > dim - offset || new_y < offset || new_y > dim - offset) {
                continue;
            }
            
            let new_pos = vec2<i32>(new_x, new_y);
            let compare_value = textureLoad(itex_1, new_pos).r;
            
            if compare_value != loaded_value{
                return 1.0;
            }
        }
    }
    return 0.0;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let x = global_id.x;
    let y = global_id.y;
    let upos = vec2<i32>(i32(x), i32(y));

    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let current_1 = textureLoad(itex_1, upos);

    // returns 1 if any of the neighbors are different
    let edge = test_neighbors(i32(x), i32(y), true, current_1.r);
    
    // store the edges in the grid in case they come in handy
    grid_a.ints[x][y][0] = i32(edge);
    
    // for the jump flood algorithm we set the distance to "infinity" for each texel, except for
    // the boundaries which we set to 0
    let inverted_scaled_edge = (1. - edge) * 1000000.0;
    let initial_step_value = 512.;
    
    textureStore(otex_1, upos, vec4f(inverted_scaled_edge, initial_step_value, 0., 1.));
    textureStore(otex_2,upos, textureLoad(itex_2,upos)); // todo test using the storage buffer to avoid constantly swapping textures
    grid_a.floats[x][y][0] = current_1.r;
}