use bevy::
    prelude::*
;
use gpu_reaback::{ GpuReadbackPlugin, MainWorldReceiver};

mod gpu_reaback;

const TEXTURE_SIZE: usize = 20;



fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, GpuReadbackPlugin))
        .add_systems(Update, receive)
        .run();
}

pub fn receive(receiver: Res<MainWorldReceiver>) {
    if let Ok(data) = receiver.try_recv() {
        // Print a visual representation
        let size = 20; // Match this with your uniforms size
        println!("");
        for y in 0..size {
            let mut line = String::new();
            for x in 0..size {
                let value = data[y * size + x as usize];
                line.push(if value > 0.5 { '█' } else { '0' });
            }
            println!("{}", line);
        }
    }
}
