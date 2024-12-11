use bevy::prelude::*;


pub const COMMON_HANDLE: Handle<Shader> = Handle::weak_from_u128(13278847158748079035);
pub const EXTRACT_HANDLE: Handle<Shader> = Handle::weak_from_u128(33378847158248049035);
pub const UTIL_NOISE_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(14378847158248049035);
pub const UTILS_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(25378847158248049035);

// The length of the buffer sent to the gpu
pub const BUFFER_LEN: usize = 1024;
pub const GRID_SIZE: usize = 8;