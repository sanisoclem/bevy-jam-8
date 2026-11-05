use crate::{PlayerCamera, orbit::CameraOrbit};
use bevy::prelude::*;
use serde::Deserialize;
use std::f32::consts::{PI, TAU};

const HEAD_TURN_START: f32 = 0.2;

pub fn walk_plugin(app: &mut App) {
  app
    .add_observer(on_walk_camera)
    .add_systems(Update, advance_walk);
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct CameraPose {
  pub eye: Vec3,
  pub target: Vec3,
}

impl CameraPose {
  pub fn transform(self) -> Transform {
    Transform::from_translation(self.eye).looking_at(self.target, Vec3::Y)
  }
}

#[derive(Event, Debug)]
pub struct WalkCamera {
  pub to: CameraPose,
  pub secs: f32,
}

#[derive(Component, Debug)]
pub struct CameraWalk {
  from: Transform,
  to: Transform,
  timer: Timer,
}

impl CameraWalk {
  fn transform(&self) -> Transform {
    let progress = self.timer.fraction();
    let head_turn = (progress - HEAD_TURN_START) / (1.0 - HEAD_TURN_START);
    let stride = self
      .from
      .translation
      .lerp(self.to.translation, eased(progress));
    Transform {
      translation: stride + Vec3::Y * step_bob(progress),
      rotation: self.from.rotation.slerp(self.to.rotation, eased(head_turn)),
      ..self.to
    }
  }
}

fn eased(progress: f32) -> f32 {
  EaseFunction::SmootherStep.sample_clamped(progress)
}

fn step_bob(progress: f32) -> f32 {
  0.014 * (progress * TAU * 3.0).sin() * (progress * PI).sin()
}

fn on_walk_camera(
  walk: On<WalkCamera>,
  camera: Single<(Entity, &Transform, Option<&CameraOrbit>), With<PlayerCamera>>,
  mut commands: Commands,
) {
  let (entity, from, orbit) = *camera;
  let to = orbit.map_or(walk.to, |x| x.turned(walk.to));
  commands
    .entity(entity)
    .remove::<CameraOrbit>()
    .insert(CameraWalk {
      from: *from,
      to: to.transform(),
      timer: Timer::from_seconds(walk.secs, TimerMode::Once),
    });
}

fn advance_walk(
  cameras: Query<(Entity, &mut Transform, &mut CameraWalk)>,
  time: Res<Time>,
  mut commands: Commands,
) {
  for (entity, mut transform, mut walk) in cameras {
    walk.timer.tick(time.delta());
    *transform = walk.transform();
    if walk.timer.is_finished() {
      commands.entity(entity).remove::<CameraWalk>();
    }
  }
}
