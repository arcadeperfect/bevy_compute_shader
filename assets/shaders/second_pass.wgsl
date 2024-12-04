struct Params {
    dimensions: u32,
    radius: f32,
    noise_seed: u32,
    noise_scale: f32,
    noise_amplitude: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>){
  let upos = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let value = textureLoad(input_texture, upos);
    textureStore(output_texture, upos, vec4<f32>(1.0 - value.x, 1.0 - value.y, 1.0 - value.z, value.w));
}

