// #import compute::noise
// #import compute::utils
// // #import noise

// struct Params {
//     dimensions: u32,
//     radius: f32,
// }
// @group(0) @binding(0) var<uniform> params: Params;


// @group(0) @binding(1) var texture: texture_storage_2d<rgba32float, write>;

// @compute @workgroup_size(1)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

//     let x = global_id.x;
//     let y = global_id.y;
//     let dim = params.dimensions;
//     let pos = vec2<f32>(
//         f32(x) / f32(dim),
//         f32(y) / f32(dim)
//     );

//     let mult = 10.0;

//     // var seed = vec2<f32>(
//     //     pos.x *mult,
//     //     pos.y *mult
//     // );

//     let center = vec2<f32>(0.5, 0.5);
//     let angle = atan2(pos.x - center.x, pos.y - center.y);  // Get raw angle
//     let seed = vec2f(cos(angle), sin(angle)); 
//     var n = noise::noise2(seed * mult);  // mult to adjust noise frequency

//     n = utils::remap(n,-1.0,1.0,-0.01,0.01);



//     let r = params.radius * 2.0 + n;

//     let dist = distance(pos, center);
//     let upos = vec2<i32>(i32(x), i32(y)); 
//     let v = select(0.0, 1.0, dist <= r);

//     textureStore(texture, upos, vec4<f32>(v , 0., 0., 1.));    
// }

#import compute::noise
#import compute::utils

struct Params {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
}
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var texture: texture_storage_2d<rgba32float, write>;

// Changed to 8x8 workgroup size - better for most GPUs
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Early return if we're outside the texture dimensions
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let dim = params.dimensions;
    let pos = vec2<f32>(
        f32(x) / f32(dim),
        f32(y) / f32(dim)
    );

    let mult = 10.0;
    let center = vec2<f32>(0.5, 0.5);
    let angle = atan2(pos.x - center.x, pos.y - center.y);
    let seed = vec2f(cos(angle), sin(angle)); 
    var n = noise::noise2(seed * mult * params.noise_scale);

    n = utils::remap(n,-1.0,1.0,-0.01,0.01);
    n = n * params.noise_amplitude;

    let r = params.radius * 0.4 + n;
    let dist = distance(pos, center);
    let upos = vec2<i32>(i32(x), i32(y)); 
    let v = select(0.0, 1.0, dist <= r);

    textureStore(texture, upos, vec4<f32>(v , 0., 0., 1.));    
} 

