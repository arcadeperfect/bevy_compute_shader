#import compute::noise
// #import compute::utils
#import compute::anoise::{psrdnoise2, fbma, terrain_gpt, terrain_claude, terrain_corrected, generate_varied_terrain}
#import compute::common::{Params, BUFFER_LEN, STRIP_SIZE, DataGrid, DataStrip}


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


fn get_weighted_neighbor_count(x: i32, y: i32, radius: f32) -> f32 {
    var found = 0.0;
    let dim = i32(params.dimensions);
    let r = i32(ceil(radius));
    var count = 0.0;

    for(var i = -r; i <= r; i++) {
        for(var j = -r; j <= r; j++) {
            let dist_sq = f32(i * i + j * j);
            if (dist_sq > radius * radius) {
                continue;
            }
            
            if(i == 0 && j == 0) {
                continue;
            }

            let new_x = x + i;
            let new_y = y + j;
            
            if(new_x < 0 || new_x >= dim || new_y < 0 || new_y >= dim) {
                found += 1.0; // Treat out-of-bounds as walls
                continue;
            }

            let new_pos = vec2<i32>(new_x, new_y);
            let v = textureLoad(itex_2, new_pos).r;
            
            // Weight by distance from center
            let weight = 1.0 - sqrt(dist_sq) / radius;
            found += v * weight;
            count += 1.0;
        }
    }

    let normed: f32 = found / count;

    return normed;
}

fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let from_range = from_max - from_min;
    let to_range = to_max - to_min;
    return (value - from_min) * to_range / from_range + to_min;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    
    let scaled_radius = params.ca_search_radius * (8.0 / (f32(params.dimensions) / 128.0));
    let nbs = get_weighted_neighbor_count(i32(x), i32(y), scaled_radius);
    var thresh = params.ca_thresh; 
    
    let edge_dist = grid_a.floats[x][y][1];
    var weighted_thresh = thresh * pow((1-edge_dist), params.ca_edge_pow);
    thresh = mix(thresh, weighted_thresh, params.edge_suppress_mix);
    thresh = remap(thresh, 0., 1., 0.14, 0.31);


    

    var caves = select(
        0.,
        1.,
        nbs > thresh,
    );

    
    textureStore(otex_2, upos, vec4f(caves, 0., 0., 0.));

}
