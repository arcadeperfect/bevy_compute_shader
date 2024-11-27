@group(0) @binding(0)
var<storage, read_write> output: array<f32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let angle = f32(id.x) * 2.0 * 3.14159 / f32(arrayLength(&output));
    output[id.x] = sin(angle);
}