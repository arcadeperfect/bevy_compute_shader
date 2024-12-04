#import compute::noise
#import compute::utils

struct Params {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
    noise_offset: f32,
    warp_amount: f32,  // Controls the intensity of the warping
    warp_scale: f32,   // Controls the scale of the noise used for warping
}
// @group(0) @binding(0) var<uniform> params: Params;
// @group(0) @binding(1) var texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;


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
    
    var pos = vec2f(
        f32(x) / f32(dim),
        f32(y) / f32(dim)
    );


    let mult = 10.0;
    let center = vec2<f32>(0.5, 0.5);
    
    let angle = atan2(pos.x - center.x, pos.y - center.y);
    let seed = vec2f(cos(angle), sin(angle)); 
    var n = noise::fbm((seed * mult * params.noise_scale) + params.noise_offset);

    n = utils::remap(n,-1.0,1.0,-0.01,0.01);
    n = n * params.noise_amplitude;

    let r = params.radius * 0.4 + n;
    let dist = distance(pos, center);
    let upos = vec2<i32>(i32(x), i32(y)); 

    let v = select(0.0, 1.0, dist <= r);

    pos = pos - center;
    let mag = length(pos);
    pos = vec2f(mag, 0.0);
    let edge = vec2f(r, 0.0);

    textureStore(output_texture, upos, vec4<f32>(v , dist, distance(pos, edge), 1.));    

} 

