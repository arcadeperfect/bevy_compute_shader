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
    
    let pos = vec2f(f32(x), f32(y));

    // Early return if we're outside the texture dimensions
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    let current = textureLoad(input_texture, upos);
    // textureStore(output_texture, upos, current);

    
    let v = noise::rand11(f32(x * (y*y)) + params.misc_f * 100);
    // let v = noise::fbm(pos * 1000. + params.misc_f * 0.01);
    let s = select(0.,1.,v <= params.noise_weight);
    

    // store noise in r channel (bc texture buffer is optimal for cellular automata)
    // store normalized distance to edge in g channel
    // textureStore(output_texture, upos, vec4f(f32(s), current.a, 1., 0.));
    let s1 = f32(s);
    textureStore(output_texture, upos, vec4f(s1,current.a, 0.0, 1.0));
    
    // the rest gets stored in the storage buffer
    grid_a.floats[upos.x][upos.y][0] = current.r; // dist to center
    grid_a.floats[upos.x][upos.y][1] = current.g; // dist to edge
    grid_a.floats[upos.x][upos.y][2] = current.b; // deformed radius
    // input_grid.floats[upos.x][upos.y][3] = current.a; // normalized dist to edge
    // input_grid.floats[upos.x][upos.y][4] = s1; // normalized dist to edge
    
}

// r, g, and b are left as is
// noise is added to a