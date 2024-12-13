use bevy::prelude::*;

// Component to mark the camera we want to control
#[derive(Component)]
pub struct CameraController {
    pub move_speed: f32,
    pub zoom_speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 500.0,
            zoom_speed: 1.0,
        }
    }
}

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_controller);
    }
}

fn camera_controller(
    mut camera_query: Query<(&mut Transform, &CameraController)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((mut transform, controller)) = camera_query.get_single_mut() else {
        return;
    };

    // Handle keyboard input for movement
    let mut movement = Vec3::ZERO;
    let mut zoom = 0.0;
    
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        movement.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        movement.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyQ) || keyboard.pressed(KeyCode::KeyZ) {
        zoom += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyE) || keyboard.pressed(KeyCode::KeyX) {
        zoom -= 1.0;
    }

    // Normalize movement vector to prevent faster diagonal movement
    if movement != Vec3::ZERO {
        movement = movement.normalize();
    }

    // Apply movement based on time delta and speed
    transform.translation += movement * controller.move_speed * time.delta_secs();
    transform.scale += zoom * controller.zoom_speed * time.delta_secs();
}