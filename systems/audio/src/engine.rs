use bevy::{
  audio::{AddAudioSource as _, Source},
  platform::collections::HashMap,
  prelude::*,
};
use rodio::source::from_iter;
use std::{hash::Hash, marker::PhantomData};
use sys_asset::{AssetBundle, AssetLoaded, IAssetBundle, SysAssetPlugin};
use utils::{
  assets::{AssetBarrier, AssetBarrierGuard},
  easing::EasingGoal,
};

use crate::clock::{Clocked, Cue, Held, TrackClock};

pub struct AudioPlugin<L: AudioLibrary, C: AudioChannelLayout> {
  phantom: PhantomData<fn() -> (L, C)>,
}
impl<L: AudioLibrary, C: AudioChannelLayout> Default for AudioPlugin<L, C> {
  fn default() -> Self {
    Self {
      phantom: Default::default(),
    }
  }
}

impl<L: AudioLibrary, C: AudioChannelLayout> Plugin for AudioPlugin<L, C> {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(SysAssetPlugin::<AudioLibraryResource<L, C>>::default())
      .add_audio_source::<TrackAudio>()
      .add_systems(
        Update,
        (
          update_channel_volume::<L, C>,
          update_entity_volumes::<L, C>,
          update_sink_volumes::<L, C>,
        )
          .chain(),
      )
      .add_observer(process_audio_commands::<L, C>)
      .add_observer(process_assets::<L, C>);
  }
}

pub trait AudioLibrary: Sized + Hash + Copy + Sync + Send + Eq + PartialEq + 'static {
  fn load_all(asset_server: &AssetServer, guard: AssetBarrierGuard) -> HashMap<Self, AudioDef>;
}

pub trait AudioChannelLayout: Sized + Hash + Eq + PartialEq + Copy + Sync + Send + 'static {
  fn initial_state() -> HashMap<Self, AudioChannelSettings>;
}

pub enum AudioDef {
  Once(Handle<AudioSource>),
  Looped(Handle<AudioSource>),
  IntroLooped {
    intro: Handle<AudioSource>,
    main: Handle<AudioSource>,
  },
}

impl AudioDef {
  pub fn try_into_processed_audio(
    &self,
    audio_assets: &Assets<AudioSource>,
  ) -> Option<ProcessedAudio> {
    match self {
      AudioDef::Once(s) => {
        if let Some(ss) = audio_assets.get(s) {
          let processed = ProcessedAudio {
            sources: vec![ss.clone()],
            process: |sources| Box::new(sources.first().unwrap().decoder()),
            lead_in_secs: 0.0,
          };

          return Some(processed);
        }
      }
      AudioDef::Looped(s) => {
        if let Some(ss) = audio_assets.get(s) {
          let processed = ProcessedAudio {
            sources: vec![ss.clone()],
            process: |sources| Box::new(sources.first().unwrap().decoder().repeat_infinite()),
            lead_in_secs: 0.0,
          };

          return Some(processed);
        }
      }
      AudioDef::IntroLooped { intro, main } => {
        if let Some(is) = audio_assets.get(intro)
          && let Some(ms) = audio_assets.get(main)
        {
          let processed = ProcessedAudio {
            sources: vec![is.clone(), ms.clone()],
            lead_in_secs: duration_secs(is),
            process: |sources| {
              let copy: Vec<_> = sources
                .iter()
                .enumerate()
                .map(|(i, f)| -> Box<dyn Source + Send + Sync> {
                  if i == 1 {
                    Box::new(f.decoder().repeat_infinite())
                  } else {
                    Box::new(f.decoder())
                  }
                })
                .collect();
              Box::new(from_iter(copy))
            },
          };
          return Some(processed);
        }
      }
    };

    None
  }
}

fn duration_secs(source: &AudioSource) -> f64 {
  let decoder = source.decoder();
  let samples_per_sec =
    f64::from(decoder.channels().get()) * f64::from(decoder.sample_rate().get());
  decoder.count() as f64 / samples_per_sec
}

#[derive(Component, Debug)]
pub struct PlayingTrack<L: AudioLibrary>(pub L);

#[derive(Component)]
pub struct AudioController<C: AudioChannelLayout> {
  channel: C,
  volume: f32,
  current_volume_cmd: Option<EasingGoal>,
  cued_fade: Option<CuedFade>,
  despawn_on_stop: bool,
  despawn_on_zero_volume: bool,
  speed: f32,
}

struct CuedFade {
  cue: Cue,
  fade: EasingGoal,
}

impl<C: AudioChannelLayout> AudioController<C> {
  fn new(channel: C, volume: f32) -> Self {
    Self {
      channel,
      volume,
      current_volume_cmd: None,
      cued_fade: None,
      despawn_on_stop: true,
      despawn_on_zero_volume: true,
      speed: 1.0,
    }
  }

  fn fade_now(&mut self, fade: EasingGoal) {
    self.cued_fade = None;
    self.current_volume_cmd = Some(fade);
  }

  fn fade_on(&mut self, cue: Option<Cue>, fade: EasingGoal) {
    match cue {
      Some(cue) => self.cued_fade = Some(CuedFade { cue, fade }),
      None => self.fade_now(fade),
    }
  }

  fn start_cued_fade(&mut self) {
    if self.cued_fade.as_ref().is_some_and(|x| x.cue.passed()) {
      self.current_volume_cmd = self.cued_fade.take().map(|x| x.fade);
    }
  }

  fn is_settled(&self) -> bool {
    self.current_volume_cmd.is_none() && self.cued_fade.is_none()
  }
}

#[derive(Clone)]
pub struct ProcessedAudio {
  pub sources: Vec<AudioSource>,
  pub process: fn(sources: &Vec<AudioSource>) -> Box<dyn Source + Sync + Send>,
  pub lead_in_secs: f64,
}

#[derive(Asset, TypePath)]
pub struct TrackAudio {
  audio: ProcessedAudio,
  clock: TrackClock,
  start: Option<Cue>,
}

impl Decodable for TrackAudio {
  type Decoder = Box<dyn Source + Sync + Send>;
  fn decoder(&self) -> Self::Decoder {
    let clocked = Clocked::new(
      (self.audio.process)(&self.audio.sources),
      self.clock.clone(),
    );
    match &self.start {
      Some(cue) => Box::new(Held::new(clocked, cue.clone())),
      None => Box::new(clocked),
    }
  }
}

pub struct AudioLibraryResource<T, C> {
  definitions: HashMap<T, AudioDef>,
  processed: Option<HashMap<T, ProcessedAudio>>,
  channels: HashMap<C, AudioChannelSettings>,
}

#[derive(Default)]
pub struct AudioChannelSettings {
  pub volume: f32,
  pub current_vol_cmd: Option<EasingGoal>,
}

impl<T, C> IAssetBundle for AudioLibraryResource<T, C>
where
  T: AudioLibrary,
  C: AudioChannelLayout,
{
  fn load_all(asset_server: &AssetServer) -> (AssetBarrier, Self) {
    let (barrier, guard) = AssetBarrier::new();
    let defs = <T as AudioLibrary>::load_all(asset_server, guard);

    (
      barrier,
      Self {
        definitions: defs,
        processed: default(),
        channels: C::initial_state(),
      },
    )
  }
}
#[derive(Event, Debug)]
pub enum AudioCommand<L, C> {
  StopAllInChannel(C),
  ReplaceAllAndFadeInto(L, C),
  ReplaceAllOnPhrase(L, C, Crossfade),
  InsertOnce(L, C, Speed),
}

#[derive(Clone, Copy, Debug)]
pub struct Crossfade {
  pub phrase_secs: f32,
  pub fade_secs: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum Speed {
  Exact,
  Varied,
}

impl Speed {
  fn factor(self) -> f32 {
    match self {
      Speed::Exact => 1.0,
      Speed::Varied => fastrand::f32() * 0.1 + 0.9,
    }
  }
}

fn process_assets<L, C>(
  _trigger: On<AssetLoaded<AudioLibraryResource<L, C>>>,
  mut audio_lib_bundle: ResMut<AssetBundle<AudioLibraryResource<L, C>>>,
  audio_assets: Res<Assets<AudioSource>>,
) where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  let AssetBundle::Loaded(audio_lib) = audio_lib_bundle.as_mut() else {
    return;
  };
  let mut h = HashMap::new();
  for (k, v) in audio_lib.definitions.iter() {
    if let Some(p) = v.try_into_processed_audio(&audio_assets) {
      h.insert(*k, p);
    }
  }
  audio_lib.processed = Some(h);
}

pub fn process_audio_commands<L, C>(
  evt: On<AudioCommand<L, C>>,
  mut commands: Commands,
  mut qry: Query<(&mut AudioController<C>, &TrackClock)>,
  audio_lib_bundle: Res<AssetBundle<AudioLibraryResource<L, C>>>,
  asset_server: Res<AssetServer>,
) where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  let AssetBundle::Loaded(audio_lib) = audio_lib_bundle.as_ref() else {
    return;
  };
  let Some(lib) = &audio_lib.processed else {
    return;
  };

  match evt.event() {
    AudioCommand::StopAllInChannel(channel) => {
      for (mut ctl, _) in qry.iter_mut() {
        if &ctl.channel != channel {
          continue;
        }
        ctl.fade_now(EasingGoal::Instant(0.0));
      }
    }
    AudioCommand::ReplaceAllAndFadeInto(to_play, channel) => {
      for (mut ctl, _) in qry.iter_mut() {
        if &ctl.channel != channel {
          continue;
        }
        let fade = EasingGoal::fade(ctl.volume, 0.0, 3.0);
        ctl.fade_now(fade);
      }
      let Some(audio) = lib.get(to_play) else {
        return;
      };
      let fading_in = AudioController {
        current_volume_cmd: Some(EasingGoal::fade(0.1, 1.0, 3.0)),
        ..AudioController::new(*channel, 0.1)
      };
      spawn_track(
        &mut commands,
        &asset_server,
        audio,
        *to_play,
        fading_in,
        None,
      );
    }
    AudioCommand::ReplaceAllOnPhrase(to_play, channel, crossfade) => {
      let Some(audio) = lib.get(to_play) else {
        return;
      };
      let cue = loudest_clock(&qry, *channel)
        .map(|x| Cue::on_next_phrase(x, audio.lead_in_secs, f64::from(crossfade.phrase_secs)));
      for (mut ctl, _) in qry.iter_mut() {
        if &ctl.channel != channel {
          continue;
        }
        let fade = EasingGoal::ease(ctl.volume, 0.0, crossfade.fade_secs, EaseFunction::SineIn);
        ctl.fade_on(cue.clone(), fade);
      }
      let mut fading_in = AudioController::new(*channel, 0.0);
      fading_in.fade_on(
        cue.clone(),
        EasingGoal::ease(0.0, 1.0, crossfade.fade_secs, EaseFunction::SineOut),
      );
      spawn_track(
        &mut commands,
        &asset_server,
        audio,
        *to_play,
        fading_in,
        cue,
      );
    }
    AudioCommand::InsertOnce(to_play, channel, speed) => {
      let Some(audio) = lib.get(to_play) else {
        return;
      };
      let controller = AudioController {
        speed: speed.factor(),
        ..AudioController::new(*channel, 1.0)
      };
      spawn_track(
        &mut commands,
        &asset_server,
        audio,
        *to_play,
        controller,
        None,
      );
    }
  };
}

fn loudest_clock<C: AudioChannelLayout>(
  tracks: &Query<(&mut AudioController<C>, &TrackClock)>,
  channel: C,
) -> Option<TrackClock> {
  tracks
    .iter()
    .filter(|(x, _)| x.channel == channel)
    .max_by(|(a, _), (b, _)| a.volume.total_cmp(&b.volume))
    .map(|(_, x)| x.clone())
}

fn spawn_track<L: AudioLibrary, C: AudioChannelLayout>(
  commands: &mut Commands,
  asset_server: &AssetServer,
  audio: &ProcessedAudio,
  track: L,
  controller: AudioController<C>,
  start: Option<Cue>,
) {
  let clock = TrackClock::default();
  let handle = asset_server.add(TrackAudio {
    audio: audio.clone(),
    clock: clock.clone(),
    start,
  });
  commands.spawn((AudioPlayer(handle), PlayingTrack(track), controller, clock));
}

pub fn update_channel_volume<L, C>(
  mut audio_lib_bundle: ResMut<AssetBundle<AudioLibraryResource<L, C>>>,
  time: Res<Time<Real>>,
) where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  let AssetBundle::Loaded(audio_lib) = audio_lib_bundle.as_mut() else {
    return;
  };
  for ctl in audio_lib.channels.values_mut() {
    let Some(goal) = &mut ctl.current_vol_cmd else {
      continue;
    };
    let step = goal.advance(time.delta());
    ctl.volume = step.value;
    if step.finished {
      ctl.current_vol_cmd = None;
    }
  }
}

pub fn update_entity_volumes<L, C>(mut qry: Query<&mut AudioController<C>>, time: Res<Time<Real>>)
where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  for mut ctl in qry.iter_mut() {
    ctl.start_cued_fade();
    let Some(goal) = &mut ctl.current_volume_cmd else {
      continue;
    };
    let step = goal.advance(time.delta());
    ctl.volume = step.value;
    if step.finished {
      ctl.current_volume_cmd = None;
    }
  }
}

pub fn update_sink_volumes<L, C>(
  mut cmd: Commands,
  audio_lib_bundle: Res<AssetBundle<AudioLibraryResource<L, C>>>,
  mut qry: Query<(Entity, &mut AudioSink, &AudioController<C>)>,
) where
  L: AudioLibrary,
  C: AudioChannelLayout,
{
  let AssetBundle::Loaded(audio_lib) = audio_lib_bundle.as_ref() else {
    return;
  };
  for (e, mut sink, ctl) in qry.iter_mut() {
    let Some(channel_settings) = audio_lib.channels.get(&ctl.channel) else {
      continue;
    };

    sink.set_volume(bevy::audio::Volume::Linear(
      channel_settings.volume * ctl.volume,
    ));
    sink.set_speed(ctl.speed);

    if (sink.empty() && ctl.despawn_on_stop)
      || (sink.volume().to_linear() <= 0.01 && ctl.despawn_on_zero_volume && ctl.is_settled())
    {
      cmd.entity(e).despawn();
    }
  }
}
