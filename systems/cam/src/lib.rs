use bevy::{
  core_pipeline::tonemapping::{DebandDither, Tonemapping},
  post_process::bloom::Bloom,
  prelude::*,
};

pub struct SysCamPlugin;

impl Plugin for SysCamPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup_camera)
      .add_systems(FixedUpdate, update_camera);
  }
}

#[derive(Component, Debug)]
pub struct PlayerCamera;

#[derive(Component, Debug)]
pub struct CameraTarget;

fn setup_camera(mut cmd: Commands) {
  cmd.spawn((
    Camera2d,
    Camera {
      clear_color: ClearColorConfig::Custom(Color::BLACK),
      ..default()
    },
    Transform::default(),
    Tonemapping::TonyMcMapface,
    Bloom::default(),
    DebandDither::Enabled,
    PlayerCamera,
  ));
}

const CAMERA_DECAY_RATE: f32 = 2.;
fn update_camera(
  mut camera: Single<&mut Transform, (With<Camera2d>, Without<CameraTarget>)>,
  target: Single<&Transform, (With<CameraTarget>, Without<Camera2d>)>,
  time: Res<Time>,
) {
  let Vec3 { x, y, .. } = target.translation;
  let direction = Vec3::new(x, y, camera.translation.z);

  camera
    .translation
    .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
