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

/*
Determine the edge of the planet by comparing the warped radius against the distance field
Subtract the gaves
*/


@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Early return if we're outside the texture dimensions
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));    
    let dim = f32(params.dimensions);

    let current_1 = textureLoad(itex_1, upos);
    let current_2 = textureLoad(itex_2, upos);
    
    let caves = current_1.r;
    let normalized_distance_to_edge = current_2.a;
    let deformed_radius = current_2.b;
    
    var rock = select(1.0, 0.0, normalized_distance_to_edge < deformed_radius);
    rock = rock - caves;
    rock = clamp(rock, 0., 1.);
    
    textureStore(otex_1, upos, vec4f(rock, 0., 0., 1.));
    textureStore(otex_2,upos, textureLoad(itex_2,upos)); // todo test using the storage buffer to avoid constantly swapping textures

}