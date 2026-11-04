use bevy::{platform::collections::HashMap, prelude::*};
use macros::AudioLibrary;

mod clock;
mod engine;

pub use clock::TrackClock;
pub use engine::*;

pub struct SysAudioPlugin;

impl Plugin for SysAudioPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(AudioPlugin::<GameAudioLibrary, GameAudioChannels>::default());
  }
}

pub type GameAudioCommand = AudioCommand<GameAudioLibrary, GameAudioChannels>;
pub type GameAudioAssets = AudioLibraryResource<GameAudioLibrary, GameAudioChannels>;

#[derive(AudioLibrary, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum GameAudioLibrary {
  #[looped("menu.ogg")]
  Menu,
  #[intro_looped("audio/game_intro.ogg", "audio/game_loop.ogg")]
  Lofi,
  #[once("audio/POWERUP-A9DDF1B8730ECF3C.ogg")]
  ButtonEffect,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum GameAudioChannels {
  Music,
  Effects,
  UI,
}
impl AudioChannelLayout for GameAudioChannels {
  fn initial_state() -> HashMap<Self, AudioChannelSettings> {
    [Self::Music, Self::Effects, Self::UI]
      .into_iter()
      .map(|channel| {
        (
          channel,
          AudioChannelSettings {
            volume: 1.0,
            ..default()
          },
        )
      })
      .collect()
  }
}
