use bevy::{math::StableInterpolate, prelude::*};
use std::f32::consts::LN_2;

pub struct SysAnimPlugin;

impl Plugin for SysAnimPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(PostUpdate, animate.before(TransformSystems::Propagate));
  }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct AnimateTo {
  pub to: Transform,
  pub secs: f32,
}

fn animate(time: Res<Time>, animated: Query<(&AnimateTo, &mut Transform)>) {
  let delta = time.delta_secs();
  for (AnimateTo { to, secs }, mut transform) in animated {
    let rate = LN_2 / secs.max(0.001);
    transform
      .translation
      .smooth_nudge(&to.translation, rate, delta);
    transform.rotation.smooth_nudge(&to.rotation, rate, delta);
    transform.scale.smooth_nudge(&to.scale, rate, delta);
  }
}
