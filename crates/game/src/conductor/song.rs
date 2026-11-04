use super::{
  ConductorSettings,
  instruments::{Instruments, NoteEnvelope},
  score::{Score, Strike},
};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Resource, Debug)]
pub struct MenuSong {
  pub score: Handle<Score>,
  pub instruments: Handle<Instruments>,
}

#[derive(SystemParam)]
pub struct LoadedSong<'w> {
  song: Res<'w, MenuSong>,
  scores: Res<'w, Assets<Score>>,
  instruments: Res<'w, Assets<Instruments>>,
  settings: Res<'w, ConductorSettings>,
}

impl LoadedSong<'_> {
  pub fn get(&self) -> Option<Song<'_>> {
    Some(Song {
      score: self.scores.get(&self.song.score)?,
      instruments: self.instruments.get(&self.song.instruments)?,
      envelope: self.settings.notes,
    })
  }
}

pub struct Song<'a> {
  score: &'a Score,
  instruments: &'a Instruments,
  envelope: NoteEnvelope,
}

#[derive(Clone, Copy, Debug)]
pub struct SongSnapshot {
  pub position: f32,
  pub strike: Strike,
  pub intensity: f32,
  pub glow: f32,
  pub loudest_note: f32,
  pub volume: f32,
}

impl SongSnapshot {
  pub fn tick(&self) -> f32 {
    (self.position / 0.125).floor()
  }
}

impl Song<'_> {
  pub fn snapshot(&self, clock_secs: f64, volume: f32) -> SongSnapshot {
    let position = clock_secs.rem_euclid(f64::from(self.score.length_secs())) as f32;
    SongSnapshot {
      position,
      strike: self.score.strike(position),
      intensity: self.score.intensity(position),
      glow: self.score.glow(position),
      loudest_note: self.instruments.loudest_note(position, self.envelope),
      volume,
    }
  }

  pub fn key_levels(&self, instrument: &str, position: f32) -> Vec<f32> {
    self
      .instruments
      .key_levels(instrument, position, self.envelope)
  }

  pub fn key_count(&self) -> usize {
    self.instruments.key_count()
  }
}
