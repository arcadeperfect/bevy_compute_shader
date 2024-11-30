// struct BlurUniforms {
//     size: u32,
//     radius: f32,
// }

// @group(0) @binding(0)
// var<storage, read> input_buffer: array<f32>;

// @group(0) @binding(1)
// var<storage, read_write> output_buffer: array<f32>;

// @group(0) @binding(2)
// var<uniform> uniforms: BlurUniforms;

// @compute @workgroup_size(8, 8, 1)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let x = global_id.x;
//     let y = global_id.y;
    
//     // Early exit if we're outside the bounds
//     if (x >= uniforms.size || y >= uniforms.size) {
//         return;
//     }

//     let center_idx = y * uniforms.size + x;
//     var sum: f32 = 0.0;
//     var weight_sum: f32 = 0.0;
//     // let radius = i32(uniforms.radius);
//     let radius = 5;

//     // Gaussian blur kernel
//     for (var i = -radius; i <= radius; i++) {
//         for (var j = -radius; j <= radius; j++) {
//             let sample_x = i32(x) + i;
//             let sample_y = i32(y) + j;
            
//             // Skip samples outside the bounds
//             if (sample_x < 0 || sample_x >= i32(uniforms.size) || 
//                 sample_y < 0 || sample_y >= i32(uniforms.size)) {
//                 continue;
//             }

//             let sample_idx = u32(sample_y) * uniforms.size + u32(sample_x);
            
//             // Gaussian weight calculation
//             let distance = f32(i * i + j * j);
//             let weight = exp(-distance / (2.0 * uniforms.radius * uniforms.radius));
            
//             sum += input_buffer[sample_idx] * weight;
//             weight_sum += weight;
//         }
//     }

//     output_buffer[center_idx] = sum / weight_sum;
// }

struct BlurUniforms {
    size: u32,
    radius: f32,
}

@group(0) @binding(0)
var<storage, read> input_buffer: array<f32>;

@group(0) @binding(1)
var<storage, read_write> output_buffer: array<f32>;

@group(0) @binding(2)
var<uniform> uniforms: BlurUniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Early exit if we're outside the bounds
    if (x >= uniforms.size || y >= uniforms.size) {
        return;
    }

    let idx = y * uniforms.size + x;
    output_buffer[idx] = input_buffer[idx];
}