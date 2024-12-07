struct Params {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
    noise_offset: f32,
    warp_amount: f32,
    warp_scale: f32, 
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
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    
    let current = textureLoad(input_texture, upos);

    let dist_to_center = input_grid.floats[x][y][0];
    let deformed_radius = input_grid.floats[x][y][2];

    var v = select(0., 1., dist_to_center <= deformed_radius);
    let caves = current.r;
    v = v - caves;
   
    textureStore(output_texture, upos, vec4f(v, v, v, 1.0));

}

// r contains generated terrain
// g contains distance field from center
// b contains distance field from edges
// a is the deformed radius