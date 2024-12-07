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

fn get_weighted_neighbor_count(x: i32, y: i32, radius: f32) -> f32 {
    var found = 0.0;
    let dim = i32(params.dimensions);
    let r = i32(ceil(radius));

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
        }
    }
    return found;
}



@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;

// @compute @workgroup_size(16, 16)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let x = global_id.x;
//     let y = global_id.y;
    
//     if (x >= params.dimensions || y >= params.dimensions) {
//         return;
//     }

//     let upos = vec2<i32>(i32(x), i32(y));
//     let current = textureLoad(input_texture, upos);

//     let nze = current.a;
    
//     let scaled_radius = params.ca_search_radius * (8.0 / (f32(params.dimensions) / 128.0));
//     let neighbor_weight = get_weighted_neighbor_count(i32(x), i32(y), scaled_radius);
    
//     let base_threshold = params.ca_thresh * (scaled_radius / 4.0);
    
//     // Increased stability bias for more persistence
//     let stability_bias = 0.25;
//     let threshold = select(
//         base_threshold,
//         base_threshold * (1.0 - stability_bias),
//         nze > 0.5
//     );

//     // Wider transition zone
//     let transition_width = 0.2 * base_threshold;
//     let diff = neighbor_weight - threshold;
    
//     // Use the transition value directly instead of as a selector
//     let transition = 1.0 - smoothstep(-transition_width, transition_width, diff);
    
//     // Strong bias toward maintaining current state unless transition is significant
//     let persistence = 0.7;
//     let v = select(
//         transition,
//         nze,
//         abs(nze - transition) < persistence
//     );

//     var result = current.x - nze;
//     result = clamp(result, 0., 1.);

//     textureStore(output_texture, upos, vec4f(result, current.y, current.z, nze));
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
    
    let nze = current.a;

    // let result = nze;

    // let caves = 1.0;

    let nbs = get_weighted_neighbor_count(i32(x), i32(y), params.ca_search_radius);

    var caves = select(
        0.,
        1.,
        nbs > params.ca_thresh
    );

    var result = current.r - caves;

    // // result = current.z - result;
    // result = clamp (result, 0., 1.);

    textureStore(output_texture, upos, vec4f(current.x, current.y, current.z, caves));
}