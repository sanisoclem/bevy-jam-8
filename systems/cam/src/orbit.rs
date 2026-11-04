use crate::{CameraPose, PlayerCamera};
use bevy::prelude::*;

pub fn orbit_plugin(app: &mut App) {
  app
    .add_observer(on_orbit_camera)
    .add_systems(Update, advance_orbit);
}

#[derive(Event, Debug)]
pub struct OrbitCamera {
  pub pose: CameraPose,
  pub radians_per_sec: f32,
  pub start_angle: f32,
}

#[derive(Component, Debug)]
pub struct CameraOrbit {
  pose: CameraPose,
  radians_per_sec: f32,
  angle: f32,
}

impl CameraOrbit {
  pub fn turned(&self, pose: CameraPose) -> CameraPose {
    let turn = Quat::from_rotation_y(self.angle);
    let pivot = self.pose.target;
    CameraPose {
      eye: pivot + turn * (pose.eye - pivot),
      target: pivot + turn * (pose.target - pivot),
    }
  }
}

fn on_orbit_camera(
  orbit: On<OrbitCamera>,
  camera: Single<Entity, With<PlayerCamera>>,
  mut commands: Commands,
) {
  commands.entity(*camera).insert(CameraOrbit {
    pose: orbit.pose,
    radians_per_sec: orbit.radians_per_sec,
    angle: orbit.start_angle,
  });
}

fn advance_orbit(cameras: Query<(&mut Transform, &mut CameraOrbit)>, time: Res<Time>) {
  for (mut transform, mut orbit) in cameras {
    orbit.angle += orbit.radians_per_sec * time.delta_secs();
    *transform = orbit.turned(orbit.pose).transform();
  }
}
