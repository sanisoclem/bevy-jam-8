use bevy::prelude::*;
use rodio::{ChannelCount, Sample, SampleRate, Source};
use std::{
  sync::{
    Arc,
    atomic::{AtomicU32, AtomicU64, Ordering::Relaxed},
  },
  time::Duration,
};

#[derive(Component, Clone, Default)]
pub struct TrackClock(Arc<SampleCount>);

#[derive(Default)]
struct SampleCount {
  played: AtomicU64,
  per_sec: AtomicU32,
}

impl TrackClock {
  pub fn secs(&self) -> f64 {
    let per_sec = self.0.per_sec.load(Relaxed);
    if per_sec == 0 {
      return 0.0;
    }
    self.0.played.load(Relaxed) as f64 / f64::from(per_sec)
  }
}

#[derive(Clone)]
pub(crate) struct Cue {
  clock: TrackClock,
  at_secs: f64,
}

impl Cue {
  pub(crate) fn on_next_phrase(clock: TrackClock, lead_in_secs: f64, phrase_secs: f64) -> Self {
    let phrases = ((clock.secs() + lead_in_secs) / phrase_secs).ceil();
    Self {
      at_secs: phrases * phrase_secs - lead_in_secs,
      clock,
    }
  }

  pub(crate) fn passed(&self) -> bool {
    self.late_secs() >= 0.0
  }

  fn late_secs(&self) -> f64 {
    self.clock.secs() - self.at_secs
  }
}

pub(crate) struct Clocked<S> {
  source: S,
  clock: TrackClock,
}

impl<S: Source> Clocked<S> {
  pub(crate) fn new(source: S, clock: TrackClock) -> Self {
    let per_sec = u32::from(source.channels().get()) * source.sample_rate().get();
    clock.0.per_sec.store(per_sec, Relaxed);
    Self { source, clock }
  }
}

impl<S: Source> Iterator for Clocked<S> {
  type Item = Sample;

  fn next(&mut self) -> Option<Sample> {
    let sample = self.source.next()?;
    self.clock.0.played.fetch_add(1, Relaxed);
    Some(sample)
  }
}

impl<S: Source> Source for Clocked<S> {
  fn current_span_len(&self) -> Option<usize> {
    self.source.current_span_len()
  }

  fn channels(&self) -> ChannelCount {
    self.source.channels()
  }

  fn sample_rate(&self) -> SampleRate {
    self.source.sample_rate()
  }

  fn total_duration(&self) -> Option<Duration> {
    self.source.total_duration()
  }
}

pub(crate) struct Held<S> {
  source: S,
  cue: Cue,
  waiting: bool,
  silent_samples: u64,
}

impl<S: Source> Held<S> {
  pub(crate) fn new(source: S, cue: Cue) -> Self {
    Self {
      source,
      cue,
      waiting: true,
      silent_samples: 0,
    }
  }

  fn at_frame_start(&self) -> bool {
    self
      .silent_samples
      .is_multiple_of(u64::from(self.source.channels().get()))
  }

  fn catch_up(&mut self) {
    let late_frames = (self.cue.late_secs() * f64::from(self.source.sample_rate().get())).round();
    let late_samples = late_frames as usize * usize::from(self.source.channels().get());
    self.source.by_ref().take(late_samples).for_each(drop);
  }
}

impl<S: Source> Iterator for Held<S> {
  type Item = Sample;

  fn next(&mut self) -> Option<Sample> {
    if self.waiting && self.at_frame_start() && self.cue.passed() {
      self.waiting = false;
      self.catch_up();
    }
    if self.waiting {
      self.silent_samples += 1;
      return Some(0.0);
    }
    self.source.next()
  }
}

impl<S: Source> Source for Held<S> {
  fn current_span_len(&self) -> Option<usize> {
    self.source.current_span_len()
  }

  fn channels(&self) -> ChannelCount {
    self.source.channels()
  }

  fn sample_rate(&self) -> SampleRate {
    self.source.sample_rate()
  }

  fn total_duration(&self) -> Option<Duration> {
    self.source.total_duration()
  }
}
