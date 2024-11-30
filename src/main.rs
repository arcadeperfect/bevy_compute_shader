use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use events::CircleSizeChanged;
use gpu_compute::ComputeSettings;
use gpu_compute::{CircleSettings, GpuReadbackPlugin, MainWorldReceiver};

use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    texture::Image,
};

mod events;
mod gpu_compute;

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
        .add_event::<CircleSizeChanged>()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            GpuReadbackPlugin::new(INITIAL_SIZE, INITIAL_RADIUS),
        ))
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, (spawn_texture, spawn_camera))
        .add_systems(Update, (receive, ui_system))
        .run();
}

fn ui_system(
    mut contexts: EguiContexts,
    mut size_events: EventWriter<CircleSizeChanged>,
    current_settings: Res<CircleSettings>,
    mut compute_settings: ResMut<ComputeSettings>,
) {
    egui::SidePanel::left("control_panel")
        .resizable(true)
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Circle Controls");

            let mut size = current_settings.size as i32;
            let mut radius = current_settings.radius;

            ui.add_space(20.0);

            ui.group(|ui| {
                ui.label("Size");
                if ui
                    .add(egui::Slider::new(&mut size, 400..=1000).text("pixels"))
                    .changed()
                {
                    size_events.send(CircleSizeChanged {
                        new_size: size as u32,
                        new_radius: radius,
                    });
                }
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("Radius");
                if ui
                    .add(egui::Slider::new(&mut radius, 0.01..=0.5).text("ratio"))
                    .changed()
                {
                    size_events.send(CircleSizeChanged {
                        new_size: size as u32,
                        new_radius: radius,
                    });
                }
            });

            ui.add_space(20.0);

            ui.label(format!("Current Size: {}", size));
            ui.label(format!("Current Radius: {:.3}", radius));

            // Add GPU control toggles
            ui.add_space(20.0);
            ui.heading("GPU Controls");
            ui.add_space(10.0);

            ui.group(|ui| {
                // ui.checkbox(&mut compute_settings.enable_compute, "Enable Compute");
                // if compute_settings.enable_compute {
                //     ui.checkbox(&mut compute_settings.enable_readback, "Enable GPU Readback");
                // } else {
                //     compute_settings.enable_readback = false;
                // }

                // ui.add_space(5.0);
                // if !compute_settings.enable_compute {
                //     ui.label("⚠️ Compute shader disabled");
                // } else if !compute_settings.enable_readback {
                //     ui.label("⚠️ GPU readback disabled");
                // }

                ui.checkbox(&mut compute_settings.enable_compute, "enable compute");
                ui.checkbox(&mut compute_settings.enable_readback, "enable readback");
                

            });
        });
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
        images.insert(&texture.handle, new_image);
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
