use super::{
  ConductorSettings, MenuTrack,
  score::StrikeType,
  song::{Song, SongSnapshot},
};
use crate::objects::{
  BANDS_PER_LEG, FORECAST_SAMPLES, FORECAST_STEP_SECS, ShadowWaves, SkyMotion, Snowfall,
};
use bevy::prelude::*;
use sys_style::Lens;

pub fn sky_motion(snapshot: &SongSnapshot) -> SkyMotion {
  let since = snapshot.strike.since_secs;
  let first_flash = (-since / 0.09).exp();
  // don't flash on an accent
  let flash = match snapshot.strike.strike_type {
    StrikeType::Bar if since >= 0.17 => first_flash.max(0.7 * (-(since - 0.17) / 0.09).exp()),
    _ => first_flash,
  };
  SkyMotion {
    since_strike: since,
    strike_number: snapshot.strike.number as u32,
    flash: flash * 0.5.lerp(1.0, snapshot.intensity) * snapshot.volume,
    intensity: snapshot.intensity,
    activity: snapshot.volume,
    star_shine: snapshot.loudest_note,
    star_tick: snapshot.tick(),
  }
}

pub fn snowfall(
  settings: &ConductorSettings,
  song: &Song,
  track: &MenuTrack,
  flash: f32,
) -> Snowfall {
  let step_secs64 = f64::from(FORECAST_STEP_SECS);
  let first_secs = ((track.clock_secs - 0.75) / step_secs64).floor() * step_secs64;
  let calulate_strength = |x: usize| {
    let secs = first_secs + x as f64 * step_secs64;
    let snapshot = song.snapshot(secs, track.volume);
    settings
      .quiet_snow
      .lerp(1.0, snapshot.intensity * snapshot.volume)
  };
  Snowfall {
    strength: std::array::from_fn::<f32, FORECAST_SAMPLES, _>(calulate_strength),
    first_sample_age_secs: (track.clock_secs - first_secs) as f32,
    flash,
  }
}

pub fn increase_fire(settings: &ConductorSettings, snapshot: &SongSnapshot) -> f32 {
  let strike = (-snapshot.strike.since_secs / 0.5).exp();
  let swell = snapshot.intensity + settings.strike_flare * strike;
  (swell * snapshot.volume).min(1.0)
}

pub fn winamplification(
  settings: &ConductorSettings,
  song: &Song,
  snapshot: &SongSnapshot,
) -> ShadowWaves {
  let levels = settings.leg_instruments.each_ref().map(|x| {
    let key_levels = song.key_levels(x, snapshot.position);
    std::array::from_fn::<f32, BANDS_PER_LEG, _>(|y| {
      key_levels.get(y).map_or(0.0, |z| z * snapshot.volume)
    })
  });
  let since = snapshot.strike.since_secs;
  let strike = settings.strike_boost * (since / 0.03).min(1.0) * (-since / 0.35).exp();
  ShadowWaves {
    levels,
    band_count: song.key_count().min(BANDS_PER_LEG),
    strike_level: strike * snapshot.volume,
    limit_scale: 1.0_f32.lerp(settings.loudest_warp_limit, snapshot.intensity),
  }
}

pub fn pot_glow(settings: &ConductorSettings, snapshot: &SongSnapshot) -> f32 {
  snapshot.glow * settings.quietest_glow.lerp(1.0, snapshot.intensity) * snapshot.volume
}

pub fn lost_focus(settings: &ConductorSettings, snapshot: &SongSnapshot) -> Lens {
  let since = snapshot.strike.since_secs;
  let strength = (since / 0.07).min(1.0) * snapshot.volume;
  let hunting = 0.8 + 0.2 * (since * 7.0).cos();
  let wobble = Vec2::new((since * 47.0).sin(), (since * 61.0 + 1.7).sin());
  Lens {
    blur: settings.focus_blur * strength * hunting * (-since / 0.7).exp(),
    shake: wobble * settings.focus_shake * strength * (-since / 0.3).exp(),
  }
}
