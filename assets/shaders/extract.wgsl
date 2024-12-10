#import compute::noise
#import compute::utils

struct Params {
    dimensions: u32,

    // circle generator
    radius: f32,
    noise_seed: u32,
    noise_freq: f32,
    noise_amplitude: f32,
    nois_lacunarity:f32,
    noise_offset: f32,
    power_bias: f32,
    flatness: f32,
    steepness: f32,
    mix: f32,
    noise_warp_amount: f32,
    noise_warp_scale: f32,

    // domain warp 1
    domain_warp_1_amount_1: f32,
    domain_warp_1_scale_1: f32,
    domain_warp_1_amount_2: f32,
    domain_warp_1_scale_2: f32,
    
    // cellular automata
    noise_weight: f32,
    ca_thresh: f32,
    ca_search_radius: f32,
    ca_edge_pow: f32,
    edge_suppress_mix: f32,

    // cave domain warp
    domain_warp_2_amount_1: f32,
    domain_warp_2_scale_1: f32,
    domain_warp_2_amount_2: f32,
    domain_warp_2_scale_2: f32,
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
// @group(0) @binding(4) var<storage, read_write> output_grid: DataGrid;
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

    let dist_to_center = input_grid.floats[x][y][0];
    let deformed_radius = input_grid.floats[x][y][2];
    // let normd_dist = input_grid.floats[x][y][3];

    var v = select(0., 1., dist_to_center <= deformed_radius);
    let caves = current.r;
    v = clamp(v - caves, 0., 1.);
   

    // let c1 = vec4f(1.0, 0.0, 0.0, 1.0);
    // let c2 = vec4f(0.0, 1.0, 0.0, 1.0);
    // let c = mix(c1, c2, normd_dist);
 
    // var result = vec4f(v) * c;
    // result.a = 1.;

    // let t = u32(normd_dist * 255.); // Convert 0-1 to 0-255 for texture lookup
    let t = u32(dist_to_center * 255.); // Convert 0-1 to 0-255 for texture lookup
    var c = textureLoad(grad_texture, vec2<i32>(i32(t), 0));
    // c = pow(c, 2.0);
    c = c * c;
    var result = vec4f(v) * c;
    result.a = 1.;

    textureStore(output_texture, upos, result);

}

// r contains generated terrain
// g contains distance field from center
// b contains distance field from edges
// a is the deformed radius