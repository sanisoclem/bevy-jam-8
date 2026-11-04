use crate::{CameraPose, PlayerCamera};
use bevy::{
  camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
  prelude::*,
};

pub fn look_plugin(app: &mut App) {
  app
    .add_plugins(FreeCameraPlugin)
    .add_observer(on_look_around_camera);
}

#[derive(Event, Debug)]
pub struct LookAroundCamera(pub CameraPose);

fn on_look_around_camera(
  look: On<LookAroundCamera>,
  camera: Single<Entity, With<PlayerCamera>>,
  mut commands: Commands,
) {
  commands
    .entity(*camera)
    .insert((look.0.transform(), FreeCamera::default()));
}
