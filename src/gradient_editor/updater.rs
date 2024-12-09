use bevy::prelude::*;

use crate::Gradients;

pub fn update_gradient_texture(mut gradients:ResMut<Gradients>){

    let g = &gradients.gradient;
    
    let v = g.linear_eval(256, true);
    println!("{:?}", v)
}