use bevy::prelude::*;
use gpu_readback::{CircleSettings, CircleSizeChanged, GpuReadbackPlugin, MainWorldReceiver};

use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    texture::Image,
};

mod gpu_readback;

pub const INITIAL_SIZE: u32 = 400;
pub const INITIAL_RADIUS: f32 = 0.1;

#[derive(Resource)]
pub struct GpuTexture {
    image: Image,
    handle: Handle<Image>,
}

#[derive(Component)]
struct TextureQuad;

fn main() {
    let initial_image = Image::new(
        Extent3d {
            width: INITIAL_SIZE,
            height: INITIAL_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0u8; (INITIAL_SIZE * INITIAL_SIZE * 4) as usize], // 4 bytes per pixel for R32Float
        TextureFormat::R32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(CircleSettings {
            size: INITIAL_SIZE,
            radius: INITIAL_RADIUS,
        })
        .insert_resource(GpuTexture {
            image: initial_image,
            handle: Handle::default(),
        })
        .add_event::<CircleSizeChanged>() // Add event handler
        .add_plugins((
            DefaultPlugins,
            GpuReadbackPlugin::new(INITIAL_SIZE, INITIAL_RADIUS),
        ))
        .add_systems(Startup, (spawn_texture, spawn_camera))
        .add_systems(
            Update,
            (
                receive,
                // handle_size_changes, // Add system to handle size changes
                keyboard_input, // Example system to change size with keyboard
            ),
        )
        .run();
}
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn array_to_bevy_image(data: &[f32], size: usize) -> Image {
    let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_ne_bytes()).collect();

    return Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::R32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
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

fn receive(
    receiver: Res<MainWorldReceiver>,
    settings: Res<CircleSettings>,
    mut texture: ResMut<GpuTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Ok(data) = receiver.try_recv() {
        let size = settings.size as usize;
        let expected_len = size * size;

        if data.len() != expected_len {
            return;
        }

        let new_image = array_to_bevy_image(&data, size);
        texture.image = new_image.clone();
        
        // Fixed: Added & to borrow the handle
        images.insert(&texture.handle, new_image);

        // // Debug visualization...
        // println!("");
        // for y in 0..size {
        //     let mut line = String::new();
        //     for x in 0..size {
        //         let idx = y * size + x;
        //         line.push(' ');
        //         if idx < data.len() {
        //             let value = data[idx];
        //             line.push(if value > 0.5 { '█' } else { '0' });
        //         }
        //     }
        //     println!("{}", line);
        // }
    }
}
fn spawn_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut texture: ResMut<GpuTexture>,
) {
    let image_handle = images.add(texture.image.clone());
    texture.handle = image_handle.clone();

    commands.spawn((
        SpriteBundle {
            texture: image_handle,
            sprite: Sprite {
                custom_size: Some(Vec2::new(400.0, 400.0)), // Adjust size as needed
                ..default()
            },
            ..default()
        },
        TextureQuad,
    ));
}
