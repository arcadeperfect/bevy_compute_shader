// jump_flood_prepare

#import compute::noise
#import compute::utils
#import compute::common::{Params, BUFFER_LEN, DataGrid}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var input_texture: texture_storage_2d<rgba32float, read>;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read_write> grid_a: DataGrid;
@group(0) @binding(4) var<storage, read_write> grid_b: DataGrid;
@group(0) @binding(5) var grad_texture: texture_storage_2d<rgba32float, read>;

fn test_neighbors_thick(x: i32, y: i32) -> f32 {
    var found = 0.0;
    let dim = i32(params.dimensions);

    let ths = textureLoad(input_texture,vec2<i32>(x,y)).r;

    for(var i = -1; i < 2; i++) {
        for(var j = -1; j <= 2; j++) {
            
            if(i == 0 && j == 0) {
                continue;
            }

            let new_x = x + i;
            let new_y = y + j;
            
            if(new_x < 0 || new_x >= dim || new_y < 0 || new_y >= dim) {
                continue;
            }

            let new_pos = vec2<i32>(new_x, new_y);
            let v = textureLoad(input_texture, new_pos).r;
            
            if v != ths{
                return 1.0;
            }

        }
    }

    return 0.0;

}

fn test_neighbors_thin(x: i32, y: i32) -> f32 {
    var found = 0.0;
    let dim = i32(params.dimensions);

    let ths = textureLoad(input_texture,vec2<i32>(x,y)).r;

    if(ths == 0.){
        return 0.;
    }

    for(var i = -1; i < 2; i++) {
        for(var j = -1; j <= 2; j++) {
            
            if(i == 0 && j == 0) {
                continue;
            }

            let new_x = x + i;
            let new_y = y + j;
            
            if(new_x < 0 || new_x >= dim || new_y < 0 || new_y >= dim) {
                continue;
            }

            let new_pos = vec2<i32>(new_x, new_y);
            let v = textureLoad(input_texture, new_pos).r;
            
            if v != ths{
                return 1.0;
            }

        }
    }

    return 0.0;

}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= params.dimensions || y >= params.dimensions) {
        return;
    }

    let upos = vec2<i32>(i32(x), i32(y));
    
    let result = test_neighbors_thick(i32(x), i32(y));
    let v = (1. - result) * 1000000.0;
    textureStore(output_texture, upos, vec4f(v, 0., 0., 512.));
    grid_b.floats[x][y][0] = result;
    
}