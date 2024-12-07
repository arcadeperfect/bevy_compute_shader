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
    ca_search_radius: f32
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;

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
                // found += 1.0; // Treat out-of-bounds as walls
                continue;
            }

            let new_pos = vec2<i32>(new_x, new_y);
            let v = textureLoad(input_texture, new_pos).a;
            
            // Weight by distance from center
            let weight = 1.0 - sqrt(dist_sq) / radius;
            found += v * weight;
            count += 1.0;
        }
    }

    let normed: f32 = found / count;

    return normed;
}

// fn probb(v: f32) -> bool{

//     if(v <= 0){
//         return false;
//     }
//     if(v >= 1){
//         return true;
//     }

//     var rand = noise::rand11(f32(x * y * y));
//     return rand < v;
// }

// regarding input:
// r contains generated terrain
// g contains distance field from center
// b contains distance field from edges
// a contains weighted noise


// @compute @workgroup_size(16, 16)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let x = global_id.x;
//     let y = global_id.y;
    
//     if (x >= params.dimensions || y >= params.dimensions) {
//         return;
//     }

    

//     let upos = vec2<i32>(i32(x), i32(y));
//     let current = textureLoad(input_texture, upos);
//     let edge_dist = current.b;
//     let nze = current.a;

//     let scaled_radius = params.ca_search_radius * (8.0 / (f32(params.dimensions) / 128.0));
//     let nbs = get_weighted_neighbor_count(i32(x), i32(y), scaled_radius);

//     var thresh = params.ca_thresh;


//     // true means we are a cave and we will be subtracted from the planet

//     var selector = nbs > thresh;

//     var rand = noise::rand11(f32(x * y * y));

//     var caves = select(
//         0.,
//         1.,
//         selector
//     );
//     // caves = caves + edge_dist;

//     var result = current.r - caves;
    
//     textureStore(output_texture, upos, vec4f(current.x, current.y, current.z, caves));
// }
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    let current = textureLoad(input_texture, upos);
    let edge_dist = current.b;
    let nze = current.a;

    let scaled_radius = params.ca_search_radius * (8.0 / (f32(params.dimensions) / 128.0));
    let nbs = get_weighted_neighbor_count(i32(x), i32(y), scaled_radius);

    var thresh = params.ca_thresh;

    // Generate base cave selector from cellular automata
    var selector = nbs > thresh;
    
    // Get random value
    var rand = noise::rand11(f32(x * y * y));
    
    let seed = vec2f(f32(x), f32(y));
    var n = noise::fbm(seed * 0.01);
    n = n * 0.5 + 0.5;
    n = n - 0.3;
    n = n * 0.1;
    // Create an exponential edge falloff factor
    // pow(x, n) where n > 1 creates exponential curve
    // Higher power = sharper falloff
    let edge_power = n * 5.;  // Adjust this to control falloff sharpness
    let edge_scale = n;  // Adjust this to control where falloff starts
    let edge_factor = pow(clamp(edge_dist / edge_scale, 0.0, 1.0), edge_power);
    
    // Only allow cave formation if random value is less than edge factor
    selector = selector && (rand < edge_factor);

    var caves = select(
        0.,
        1.,
        selector
    );

    var result = current.r - caves;
    
    textureStore(output_texture, upos, vec4f(current.x, current.y, current.z, caves));
}

// r, g, and b are left as is
// caves are added to a