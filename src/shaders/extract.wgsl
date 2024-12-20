// extract

#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, DataGrid, DataStrip}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var itex_1: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var itex_2: texture_storage_2d<rgba32float, read>;
@group(0) @binding(3) var itex_3: texture_storage_2d<rgba32float, read>;
@group(0) @binding(4) var otex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(5) var<storage, read_write> grid_a: DataGrid;
@group(0) @binding(6) var<storage, read_write> grid_b: DataGrid;
@group(0) @binding(7) var<storage, read_write> strip_a: DataStrip;
@group(0) @binding(8) var<storage, read_write> strip_b: DataStrip;
@group(0) @binding(9) var grad_texture: texture_storage_2d<rgba32float, read>;


@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    
    var current_1 = textureLoad(itex_1, upos);
    // let current_2 = textureLoad(itex_2, upos);
    // let current_3 = textureLoad(itex_3, upos);

    
    // let v = strip_a.floats[0][0];
    // let v = grid_a.floats[0][0][0];
    // current_1.a = 1.0;
    // current_1.g = 1.0;



    let out = vec4f(current_1.r,current_1.g,current_1.b,1.0);

    textureStore(otex, upos, out);
}
