#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, DataGrid, DataStrip}


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

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }
    
    let upos = vec2<i32>(i32(x), i32(y)); 

    

}