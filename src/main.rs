use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::storage_buffer, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};
use crossbeam_channel::{Receiver, Sender};
use bevy::render::render_resource::TextureFormat;
use bevy::sprite::MaterialMesh2dBundle;
use gpu_reaback::{ GpuReadbackPlugin, MainWorldReceiver};

mod gpu_reaback;

const TEXTURE_SIZE: u32 = 256;



fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, GpuReadbackPlugin))
        .add_systems(Update, receive)
        .run();
}


// System that receives and processes data from the GPU
pub fn receive(receiver: Res<MainWorldReceiver>) {
    if let Ok(data) = receiver.try_recv() {
        println!("{:?}", data.len());
    }
}