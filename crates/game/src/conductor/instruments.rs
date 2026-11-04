use crate::ron_asset::RonAsset;
use bevy::prelude::*;
use serde::Deserialize;
use std::f32::consts::TAU;

const VIBRATO_FADE_SECS: f32 = 0.3;

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(from = "InstrumentsFile")]
pub struct Instruments {
  instruments: Vec<Instrument>,
  keys: Vec<f32>,
}

#[derive(Deserialize)]
struct InstrumentsFile {
  instruments: Vec<Instrument>,
}

#[derive(Deserialize, Clone, Debug)]
struct Instrument {
  name: String,
  notes: Vec<Note>,
}

// workaround: extracted raw note data from fl studio
// just like the macro score -  couldn't get spectral analysis to work reliably
// so this needs to be synced with the audio sink
#[derive(Deserialize, Clone, Copy, Debug)]
struct Note {
  secs: f32,
  key: f32,
  length: f32,
  level: f32,
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct NoteEnvelope {
  decay_secs: f32,
  vibrato_depth: f32,
}

impl RonAsset for Instruments {
  const EXTENSIONS: &'static [&'static str] = &["instruments.ron"];
}

impl From<InstrumentsFile> for Instruments {
  fn from(file: InstrumentsFile) -> Self {
    let mut instruments = file.instruments;
    for instrument in &mut instruments {
      instrument.notes.sort_by(|a, b| a.secs.total_cmp(&b.secs));
    }
    let mut keys: Vec<f32> = instruments
      .iter()
      .flat_map(|x| &x.notes)
      .map(|x| x.key)
      .collect();
    keys.sort_by(f32::total_cmp);
    keys.dedup();
    Self { instruments, keys }
  }
}

impl Instruments {
  pub fn key_count(&self) -> usize {
    self.keys.len()
  }

  pub fn key_levels(&self, name: &str, secs: f32, envelope: NoteEnvelope) -> Vec<f32> {
    let mut levels = vec![0.0; self.keys.len()];
    let Some(instrument) = self.instruments.iter().find(|x| x.name == name) else {
      return levels;
    };
    let started = instrument.notes.partition_point(|x| x.secs <= secs);
    let recent = instrument.notes[..started]
      .iter()
      .rev()
      .take_while(|x| secs - x.secs < 10.0);
    for note in recent {
      let key = self.keys.partition_point(|x| *x < note.key);
      levels[key] += envelope.level_after(*note, secs - note.secs);
    }
    levels
  }

  pub fn loudest_note(&self, secs: f32, envelope: NoteEnvelope) -> f32 {
    self
      .instruments
      .iter()
      .map(|x| {
        let started = x.notes.partition_point(|y| y.secs <= secs);
        x.notes[..started]
          .last()
          .map_or(0.0, |y| envelope.level_after(*y, secs - y.secs))
      })
      .fold(0.0, f32::max)
  }
}

impl NoteEnvelope {
  fn level_after(self, note: Note, since: f32) -> f32 {
    let attack = (since / 0.05).min(1.0);
    let decay = (-(since - note.length).max(0.0) / self.decay_secs).exp();
    let fade_in = since / VIBRATO_FADE_SECS;
    let fade_out = (note.length - since) / VIBRATO_FADE_SECS;
    let vibrato =
      self.vibrato_depth * fade_in.min(fade_out).clamp(0.0, 1.0) * (TAU * 2.0 * since).sin();
    note.level * attack * decay * (1.0 + vibrato)
  }
}
