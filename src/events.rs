use bevy::prelude::*;

// Event that gets fired when we want to change the circle size
// This lets different parts of our app react to size changes
#[derive(Event)]
pub struct CircleSizeChanged {
    pub new_size: u32,
    pub new_radius: f32,
}