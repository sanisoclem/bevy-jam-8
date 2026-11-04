use crate::{
  objects::{Blaze, LEG_COUNT, PotGlow, ShadowWaves, SkyMotion, Snowfall},
  ron_asset::ron_asset_plugin,
};
use bevy::{
  audio::{AudioSink, AudioSinkPlayback},
  ecs::system::SystemParam,
  prelude::*,
};
use instruments::{Instruments, NoteEnvelope};
use score::Score;
use serde::Deserialize;
use song::{LoadedSong, MenuSong};
use song_events::*;
use sys_audio::{GameAudioLibrary, PlayingTrack};
use sys_cam::PlayerCamera;
use sys_style::Lens;

mod instruments;
mod score;
mod song;
mod song_events;

pub struct ConductorPlugin;

impl Plugin for ConductorPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((ron_asset_plugin::<Score>, ron_asset_plugin::<Instruments>))
      .add_systems(Startup, setup)
      .add_systems(Update, conduct.run_if(resource_exists::<ConductorSettings>));
  }
}

#[derive(Resource, Deserialize, Clone, Debug)]
pub struct ConductorSettings {
  output_lag_secs: f64,
  notes: NoteEnvelope,
  leg_instruments: [String; LEG_COUNT],
  quiet_snow: f32,
  quietest_glow: f32,
  strike_flare: f32,
  strike_boost: f32,
  loudest_warp_limit: f32,
  focus_blur: f32,
  focus_shake: f32,
}

#[derive(Debug, Default)]
struct MenuTrack {
  clock_secs: f64,
  volume: f32,
}

#[derive(SystemParam)]
struct Stage<'w, 's> {
  sky: ResMut<'w, SkyMotion>,
  snowfall: ResMut<'w, Snowfall>,
  blaze: ResMut<'w, Blaze>,
  shadows: ResMut<'w, ShadowWaves>,
  pot_glow: ResMut<'w, PotGlow>,
  lenses: Query<'w, 's, &'static mut Lens, With<PlayerCamera>>,
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
  commands.insert_resource(MenuSong {
    score: asset_server.load("scores/menu.score.ron"),
    instruments: asset_server.load("scores/menu.instruments.ron"),
  });
}

fn conduct(
  tracks: Query<(&PlayingTrack<GameAudioLibrary>, &AudioSink)>,
  settings: Res<ConductorSettings>,
  time: Res<Time<Real>>,
  song: LoadedSong,
  mut track: Local<MenuTrack>,
  mut stage: Stage,
) {
  // only in the menu
  let Some((_, sink)) = tracks.iter().find(|(x, _)| x.0 == GameAudioLibrary::Menu) else {
    return;
  };
  let played_s = sink.position().as_secs_f64() - settings.output_lag_secs;
  let frame_time = time.delta_secs_f64();
  let ticked_s = track.clock_secs + frame_time;

  // audio sink time seems to be very coarse
  // WTF?
  let drift = played_s - ticked_s;
  track.clock_secs = if drift.abs() > 0.25 {
    // if too much drift, just set it
    played_s
  } else {
    // otherwise slowly ease into it so it doesn't jitter
    ticked_s + drift * (1.0 - (-frame_time / 0.2).exp())
  };
  track.volume = sink.volume().to_linear();

  let Some(song) = song.get() else {
    return;
  };

  let snapshot = song.snapshot(track.clock_secs, track.volume);
  *stage.sky = sky_motion(&snapshot);
  *stage.snowfall = snowfall(&settings, &song, &track, stage.sky.flash);
  stage.blaze.flare = increase_fire(&settings, &snapshot);
  *stage.shadows = winamplification(&settings, &song, &snapshot);
  stage.pot_glow.target = pot_glow(&settings, &snapshot);
  let focus = lost_focus(&settings, &snapshot);
  for mut lens in &mut stage.lenses {
    *lens = focus;
  }
}
