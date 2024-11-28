use bevy::prelude::*;
use gpu_reaback::{CircleUniforms, GpuReadbackPlugin, MainWorldReceiver};

mod gpu_reaback;

pub const INITIAL_SIZE: u32 = 40;
pub const INITIAL_RADIUS: f32 = 0.1;
#[derive(Resource)]
pub struct CircleSettings {
    size: u32,
    radius: f32,
}

#[derive(Event)]
pub struct CircleSizeChanged {
    new_size: u32,
    new_radius: f32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(CircleSettings {
            size: INITIAL_SIZE,
            radius: INITIAL_RADIUS,
        })
        .add_event::<CircleSizeChanged>() // Add event handler
        .add_plugins((
            DefaultPlugins,
            GpuReadbackPlugin::new(INITIAL_SIZE, INITIAL_RADIUS),
        ))
        .add_systems(
            Update,
            (
                receive,
                handle_size_changes, // Add system to handle size changes
                keyboard_input,      // Example system to change size with keyboard
            ),
        )
        .run();
}

fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut size_events: EventWriter<CircleSizeChanged>,
    current_settings: Res<CircleSettings>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        let new_size = current_settings.size + 1;
        let new_radius = current_settings.radius;
        size_events.send(CircleSizeChanged {
            new_size,
            new_radius,
        });
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        let new_size = current_settings.size - 1;
        let new_radius = current_settings.radius;
        size_events.send(CircleSizeChanged {
            new_size,
            new_radius,
        });
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        let new_size = current_settings.size;
        let new_radius = current_settings.radius + 0.01;
        size_events.send(CircleSizeChanged {
            new_size,
            new_radius,
        });
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        let new_size = current_settings.size;
        let new_radius = current_settings.radius - 0.01;
        size_events.send(CircleSizeChanged {
            new_size,
            new_radius,
        });
    }
}

fn handle_size_changes(
    mut events: EventReader<CircleSizeChanged>,
    mut settings: ResMut<CircleSettings>,
    mut uniforms: ResMut<CircleUniforms>,
) {
    for event in events.read() {
        settings.size = event.new_size;
        settings.radius = event.new_radius;
        uniforms.size = event.new_size;
        uniforms.radius = event.new_radius;
    }
}

pub fn receive(receiver: Res<MainWorldReceiver>, settings: Res<CircleSettings>) {
    if let Ok(data) = receiver.try_recv() {
        let size = settings.size as usize;
        let expected_len = size * size;

        // Check if received data matches expected size
        if data.len() != expected_len {
            return; // Skip rendering if sizes don't match
        }

        println!("");
        for y in 0..size {
            let mut line = String::new();
            for x in 0..size {
                let idx = y * size + x;
                if idx < data.len() {
                    let value = data[idx];
                    line.push(if value > 0.5 { '█' } else { '0' });
                }
            }
            println!("{}", line);
        }
    }
}
