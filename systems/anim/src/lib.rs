use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct AnimateTo {
  pub to: Transform,
  pub secs: f32,
}
