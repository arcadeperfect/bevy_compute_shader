use bevy::
    render::
        render_resource::*
    
;
use bytemuck::{Pod, Zeroable};

use crate::{BUFFER_LEN, GRID_SIZE};
use crate::constants::*;


#[derive(Debug, Clone)]
pub struct ShaderConfig {
    // shader_handle: Handle<Shader>,
    pub shader_path: &'static str,
    pub iterations: u32,
}

#[derive(Copy, Clone, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct DataGrid {
    
    // grids of dimension GRID_SIZE x GRID_SIZE but with more than 4 dimensions
    
    pub floats: [[[f32; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
    pub ints: [[[i32; GRID_SIZE]; BUFFER_LEN]; BUFFER_LEN],
}



#[derive(Copy, Clone, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct DataStrip {
    
    pub floats: [[f32; STRIP_SIZE]; STRIP_COUNT],
    pub ints: [[i32; STRIP_SIZE]; STRIP_COUNT],
}