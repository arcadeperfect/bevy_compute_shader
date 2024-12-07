#import compute::noise
#import compute::utils

struct Params {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
    noise_offset: f32,
    power_bias: f32,
    flatness: f32,
    steepness: f32,
    mix: f32,
    warp_amount: f32,
    warp_scale: f32, 
    noise_weight: f32,
    ca_thresh: f32,
    ca_search_radius: f32,
    ca_edge_pow: f32,
    edge_suppress_mix: f32
}

const BUFFER_LEN = 1024u;
struct DataGrid{
    floats: array<array<array<f32, 8>, BUFFER_LEN>, BUFFER_LEN>,
    ints: array<array<array<i32, 8>, BUFFER_LEN>, BUFFER_LEN>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read_write> input_grid: DataGrid;
@group(0) @binding(4) var<storage, read_write> output_grid: DataGrid;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Early return if we're outside the texture dimensions
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    let current = textureLoad(input_texture, upos);
    // textureStore(output_texture, upos, current);



    let pos = vec2f(f32(x), f32(y));
    let v = noise::rand11(f32(x * (y*y)));
    // let v = noise::rand11(f32(x ^ (y << 16)));
    let s = select(0.,1.,v <= params.noise_weight);
    

    // store noise in r channel (bc texture buffer is optimal for cellular automata)
    // store normalized distance to edge in g channel
    // textureStore(output_texture, upos, vec4f(f32(s), current.a, 1., 0.));
    let s1 = f32(s);
    textureStore(output_texture, upos, vec4f(s1,current.a, 0.0, 1.0));
    
    // the rest gets stored in the storage buffer
    output_grid.floats[upos.x][upos.y][0] = current.r; // dist to center
    output_grid.floats[upos.x][upos.y][1] = current.g; // dist to edge
    output_grid.floats[upos.x][upos.y][2] = current.b; // deformed radius
    
    
}

// r, g, and b are left as is
// noise is added to a