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


fn is_valid_point(p: vec2<i32>) -> bool {
    return p.x >= 0 && 
           p.y >= 0 && 
           p.x < i32(params.dimensions) && 
           p.y < i32(params.dimensions);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    
    let current_1 = textureLoad(itex_1, upos);
    
    // get current step from the g channel
    // starts at 512 and is reduced each iteration (usually 2, but can be more for better quality)
    let step = i32(current_1.g);
    if(step < 2){
        // according to log n, we should be done by now
        return;
    }

    var min_distance = current_1.r;  // This was set to a high value (ideally inf, not poss in wgsl) in the prepare pass

    // jump flood
    for(var dy = -1; dy <= 1; dy ++){
        for(var dx = -1; dx <=1; dx++) {
            let sample_pos = upos + vec2<i32>(dx, dy) * step;
            if (is_valid_point(sample_pos)) {
                let sample = textureLoad(itex_1, sample_pos);
                
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

    textureStore(otex_1, upos, vec4<f32>(
                                        min_distance, 
                                        // f32(step)/1.25, 
                                        f32(step)/2, 
                                        0.0, 
                                        1.0
                                        ));
                                        
    textureStore(otex_2,upos, textureLoad(itex_2,upos)); // todo test using the storage buffer to avoid constantly swapping textures
}