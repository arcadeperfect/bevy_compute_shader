struct Params {
    dimensions: u32,
    radius: f32,
}
@group(0) @binding(0) var<uniform> params: Params;


@group(0) @binding(1) var texture: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let dim = params.dimensions;
    let x = global_id.x;
    let y = global_id.y;
    let pos = vec2<f32>(
        f32(x) / f32(dim),
        f32(y) / f32(dim)
    );
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(pos, center);
    let upos = vec2<i32>(i32(x), i32(y)); 
    let v = select(0.0, 1.0, dist <= params.radius);

    textureStore(texture, upos, vec4<f32>(v , 0., 0., 1.));    
}
