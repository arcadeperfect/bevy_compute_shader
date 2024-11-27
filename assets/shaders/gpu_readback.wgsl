// // This shader is used for the gpu_readback example
// // The actual work it does is not important for the example

// // This is the data that lives in the gpu only buffer
// @group(0) @binding(0) var<storage, read_write> data: array<u32>;

// @compute @workgroup_size(1)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     // We use the global_id to index the array to make sure we don't
//     // access data used in another workgroup
//     data[global_id.x] += 1u;
// }

@group(0) @binding(0)
var<storage, read_write> output: array<f32>;

const TEXTURE_SIZE: u32 = 256u;
const CENTER: vec2<f32> = vec2<f32>(128.0, 128.0);
const RADIUS: f32 = 100.0;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Convert 1D buffer index to 2D coordinates
    let x = id.x;
    let y = id.y;
    let index = y * TEXTURE_SIZE + x;
    
    // Calculate position relative to center
    let pos = vec2<f32>(f32(x), f32(y));
    let dist = distance(pos, CENTER);
    
    // Set to 1.0 if inside circle, 0.0 if outside
    output[index] = select(0.0, 1.0, dist <= RADIUS);
}