// extract

#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, DataGrid}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read_write> grid_a: DataGrid;
@group(0) @binding(4) var<storage, read_write> grid_b: DataGrid;
@group(0) @binding(5) var grad_texture: texture_storage_2d<rgba32float, read>;
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    
    let current = textureLoad(input_texture, upos);
    // textureStore(output_texture, upos, current);

    // let n = current.r / i32(params.misc_i);
    let n = current.r / 1000.;

    textureStore(output_texture, upos, vec4f(n,n,n,1.0));

    // let e = grid_b.floats[x][y][0];
    // textureStore(output_texture, upos, vec4f(e,e,e,1.0));
}
