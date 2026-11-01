use bevy::prelude::*;
use std::time::Duration;

pub enum EasingGoal {
  Instant(f32),
  Fade(Timer, EasingCurve<f32>),
}

pub struct EasingStep {
  pub value: f32,
  pub finished: bool,
}

impl EasingGoal {
  pub fn fade(from: f32, to: f32, duration_secs: f32) -> Self {
    EasingGoal::Fade(
      Timer::from_seconds(duration_secs, TimerMode::Once),
      EasingCurve::new(from, to, EaseFunction::Linear),
    )
  }

  pub fn advance(&mut self, delta: Duration) -> EasingStep {
    match self {
      EasingGoal::Instant(value) => EasingStep {
        value: *value,
        finished: true,
      },
      EasingGoal::Fade(timer, curve) => {
        timer.tick(delta);
        EasingStep {
          value: curve.sample_clamped(timer.fraction()),
          finished: timer.is_finished(),
        }
      }
    }
  }
}
