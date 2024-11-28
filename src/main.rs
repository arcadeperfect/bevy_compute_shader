use bevy::{
    prelude::*, render::renderer::{RenderDevice, RenderQueue}}
;
use gpu_reaback::{ Buffers, CircleUniforms, GpuReadbackPlugin, MainWorldReceiver};

mod gpu_reaback;

const TEXTURE_SIZE: usize = 20;
const CIRCLE_RADIUS: f32 = 0.3;

#[derive(Resource)]
pub struct CircleSettings {
    size: u32,
    radius: f32,
}

impl Default for CircleSettings {
    fn default() -> Self {
        Self {
            size: TEXTURE_SIZE as u32,
            radius: CIRCLE_RADIUS,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(CircleSettings::default())
        .add_plugins((DefaultPlugins, GpuReadbackPlugin))
        .add_systems(Update, receive)
        .run();
}



pub fn receive(receiver: Res<MainWorldReceiver>, settings: Res<CircleSettings>) {
    if let Ok(data) = receiver.try_recv() {
        let size = settings.size;
        println!("");
        for y in 0..size {
            let mut line = String::new();
            for x in 0..size {
                let value = data[y as usize * (size as usize) + x as usize];
                line.push(if value > 0.5 { '█' } else { '0' });
            }
            println!("{}", line);
        }
    }
}