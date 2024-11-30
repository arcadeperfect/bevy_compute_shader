struct CircleUniforms {
    size: u32,
    radius: f32,
}

@group(0) @binding(0)
var<storage, read_write> output: array<f32>;

@group(0) @binding(1)
var<uniform> uniforms: CircleUniforms;

// Use a much larger workgroup size - 8x8 or 16x16 is typical
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Early return if we're outside the image bounds
    if (id.x >= uniforms.size || id.y >= uniforms.size) {
        return;
    }
    
    let x = id.x;
    let y = id.y;
    let index = y * uniforms.size + x;
    
    let pos = vec2<f32>(
        f32(x) / f32(uniforms.size),
        f32(y) / f32(uniforms.size)
    );
    let center = vec2<f32>(0.5, 0.5);
    
    let dist = distance(pos, center);
    output[index] = select(0.0, 1.0, dist <= uniforms.radius);
}