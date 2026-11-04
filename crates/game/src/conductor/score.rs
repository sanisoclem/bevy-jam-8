use crate::ron_asset::RonAsset;
use bevy::prelude::*;
use serde::Deserialize;
use std::num::NonZeroU32;

// macro view of the piece
// loudness analysis is ineffective as it seems
// to not coincide with the actual load/heavy parts
// probably missing something very obvbious...
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(from = "ScoreFile")]
pub struct Score {
  bars: NonZeroU32,
  secs_per_bar: f32,
  intensity: Vec<Keyframe>,
  glow: Vec<Keyframe>,
  strikes: Vec<StrikeTime>,
}

#[derive(Deserialize)]
struct ScoreFile {
  bars: NonZeroU32,
  secs_per_bar: f32,
  intensity: Vec<Keyframe>,
  accent_strikes: Vec<f32>,
  glow: Vec<Keyframe>,
}

#[derive(Deserialize, Clone, Copy, Debug)]
struct Keyframe {
  bar: f32,
  level: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrikeType {
  Bar,
  Accent,
}

#[derive(Clone, Copy, Debug)]
struct StrikeTime {
  secs: f32,
  strike_type: StrikeType,
}

#[derive(Clone, Copy, Debug)]
pub struct Strike {
  pub number: usize,
  pub strike_type: StrikeType,
  pub since_secs: f32,
}

impl RonAsset for Score {
  const EXTENSIONS: &'static [&'static str] = &["score.ron"];
}

impl From<ScoreFile> for Score {
  fn from(mut file: ScoreFile) -> Self {
    let bars = (0..file.bars.get()).map(|x| (x as f32, StrikeType::Bar));
    let accents = file.accent_strikes.iter().map(|&x| (x, StrikeType::Accent));
    let mut strikes: Vec<StrikeTime> = bars
      .chain(accents)
      .map(|(x, y)| StrikeTime {
        secs: x * file.secs_per_bar,
        strike_type: y,
      })
      .collect();
    strikes.sort_by(|a, b| a.secs.total_cmp(&b.secs));
    file.intensity.sort_by(|a, b| a.bar.total_cmp(&b.bar));
    file.glow.sort_by(|a, b| a.bar.total_cmp(&b.bar));
    Self {
      bars: file.bars,
      secs_per_bar: file.secs_per_bar,
      intensity: file.intensity,
      glow: file.glow,
      strikes,
    }
  }
}

fn level(keyframes: &[Keyframe], bar: f32) -> f32 {
  let next = keyframes.partition_point(|x| x.bar <= bar);
  let before = next.checked_sub(1).map(|x| keyframes[x]);
  match (before, keyframes.get(next)) {
    (Some(before), Some(after)) => {
      let progress = (bar - before.bar) / (after.bar - before.bar);
      before.level.lerp(after.level, progress)
    }
    (Some(keyframe), None) | (None, Some(&keyframe)) => keyframe.level,
    (None, None) => 0.0,
  }
}

impl Score {
  pub fn length_secs(&self) -> f32 {
    self.bars.get() as f32 * self.secs_per_bar
  }

  pub fn intensity(&self, secs: f32) -> f32 {
    level(&self.intensity, secs / self.secs_per_bar)
  }

  pub fn glow(&self, secs: f32) -> f32 {
    level(&self.glow, secs / self.secs_per_bar)
  }

  pub fn strike(&self, secs: f32) -> Strike {
    let next = self.strikes.partition_point(|x| x.secs <= secs);
    let number = next.saturating_sub(1);
    let current = self.strikes[number];
    Strike {
      number,
      strike_type: current.strike_type,
      since_secs: secs - current.secs,
    }
  }
}
