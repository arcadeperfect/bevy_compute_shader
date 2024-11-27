struct CircleUniforms {
    size: u32,
    radius: f32,
}

@group(0) @binding(0)
var<storage, read_write> output: array<f32>;

@group(0) @binding(1)
var<uniform> uniforms: CircleUniforms;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    let index = y * uniforms.size + x;
    
    // Calculate position relative to center (in 0-1 range)
    let pos = vec2<f32>(
        f32(x) / f32(uniforms.size),
        f32(y) / f32(uniforms.size)
    );
    let center = vec2<f32>(0.5, 0.5);
    
    // Calculate distance in 0-1 range
    let dist = distance(pos, center);
    
    // Set to 1.0 if inside circle radius (which is also in 0-1 range), 0.0 if outside
    output[index] = select(0.0, 1.0, dist <= uniforms.radius);
}