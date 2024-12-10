use bevy::prelude::*;

pub const GENERATE_CIRCLE_HANDLE: Handle<Shader> = Handle::weak_from_u128(13378847158248049035);
pub const DOMAIN_WARP_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378847158248049035);
pub const PRE_CA_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378547158240049035);
pub const CA_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378547158248049035);
pub const POST_CA_HANDLE: Handle<Shader> = Handle::weak_from_u128(23378547158248049031);
pub const EXTRACT_HANDLE: Handle<Shader> = Handle::weak_from_u128(33378847158248049035);
pub const UTIL_NOISE_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(14378847158248049035);
pub const UTIL_VECTOR_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(25378847158248049035);

// The length of the buffer sent to the gpu
pub const BUFFER_LEN: usize = 1024;
pub const GRID_SIZE: usize = 8;