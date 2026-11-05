use bevy::{
  core_pipeline::tonemapping::{DebandDither, Tonemapping},
  post_process::bloom::Bloom,
  prelude::*,
};
use std::f32::consts::FRAC_PI_3;

mod look;
mod orbit;
mod walk;

pub use look::LookAroundCamera;
pub use orbit::{CameraOrbit, OrbitCamera};
pub use walk::{CameraPose, CameraWalk, WalkCamera};

pub struct SysCamPlugin;

impl Plugin for SysCamPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((look::look_plugin, orbit::orbit_plugin, walk::walk_plugin))
      .add_systems(Startup, setup_camera);
  }
}

#[derive(Component, Debug)]
pub struct PlayerCamera;

fn setup_camera(mut cmd: Commands) {
  cmd.spawn((
    Camera3d::default(),
    Camera {
      clear_color: ClearColorConfig::Custom(Color::BLACK),
      ..default()
    },
    Projection::Perspective(PerspectiveProjection {
      fov: FRAC_PI_3,
      ..default()
    }),
    Transform::default(),
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Disabled,
    PlayerCamera,
  ));
}
