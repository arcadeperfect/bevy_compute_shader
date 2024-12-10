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
@group(0) @binding(4) var<storage, read_write> output_grid: DataGrid;
@group(0) @binding(5) var grad_texture: texture_storage_2d<rgba32float, read>;

fn test_neighbors_thick(x: i32, y: i32) -> f32 {
    var found = 0.0;
    let dim = i32(params.dimensions);

    let ths = textureLoad(input_texture,vec2<i32>(x,y)).r;

    for(var i = -1; i < 2; i++) {
        for(var j = -1; j <= 2; j++) {
            
            if(i == 0 && j == 0) {
                continue;
            }

            let new_x = x + i;
            let new_y = y + j;
            
            if(new_x < 0 || new_x >= dim || new_y < 0 || new_y >= dim) {
                continue;
            }

            let new_pos = vec2<i32>(new_x, new_y);
            let v = textureLoad(input_texture, new_pos).r;
            
            if v != ths{
                return 1.0;
            }

        }
    }

    return 0.0;

}

fn test_neighbors_thin(x: i32, y: i32) -> f32 {
    var found = 0.0;
    let dim = i32(params.dimensions);

    let ths = textureLoad(input_texture,vec2<i32>(x,y)).r;

    if(ths == 0.){
        return 0.;
    }

    for(var i = -1; i < 2; i++) {
        for(var j = -1; j <= 2; j++) {
            
            if(i == 0 && j == 0) {
                continue;
            }

            let new_x = x + i;
            let new_y = y + j;
            
            if(new_x < 0 || new_x >= dim || new_y < 0 || new_y >= dim) {
                continue;
            }

            let new_pos = vec2<i32>(new_x, new_y);
            let v = textureLoad(input_texture, new_pos).r;
            
            if v != ths{
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
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let result = test_neighbors_thin(i32(x), i32(y));
    input_grid.floats[x][y][7] = result;

}