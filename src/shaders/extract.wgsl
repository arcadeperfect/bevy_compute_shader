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

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    
    let current = textureLoad(input_texture, upos);


    let dist = current.g / current.a;
    textureStore(output_texture, upos, vec4f(dist));

    var r = current.r - current.a;
    r = clamp(r, 0., 1.);

    // textureStore(output_texture, upos, vec4f(r,r,r,1.));
    // textureStore(output_texture, upos, vec4f(r, current.g, current.b, 1.0));
    textureStore(output_texture, upos, vec4f(r, current.g, current.b, 1.0));
}

// r contains generated terrain
// g contains distance field from center
// b contains distance field from edges
// a is the deformed radius