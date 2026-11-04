use crate::{embedded_shader, style_pass::schedule_in_style_pass};
use bevy::{
  asset::{embedded_asset, embedded_path},
  core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
  ecs::{schedule::ScheduleConfigs, system::BoxedSystem},
  prelude::*,
  render::{RenderApp, extract_component::ExtractComponent, render_resource::ShaderType},
  shader::ShaderRef,
};
use serde::Deserialize;

pub fn pixel_art_plugin(app: &mut App) {
  embedded_asset!(app, "pixel_art.wesl");
  app.add_plugins(FullscreenMaterialPlugin::<PixelArt>::default());
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PixelArtSettings {
  pub pixel_lines: f32,
  pub tone_bands: f32,
  pub hue_levels: f32,
  pub outline: f32,
}

impl Default for PixelArtSettings {
  fn default() -> Self {
    Self {
      pixel_lines: 270.0,
      tone_bands: 11.0,
      hue_levels: 8.0,
      outline: 0.6,
    }
  }
}

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Debug)]
#[extract_app(RenderApp)]
pub struct PixelArt {
  pixel_lines: f32,
  tone_bands: f32,
  hue_levels: f32,
  outline: f32,
}

impl From<PixelArtSettings> for PixelArt {
  fn from(settings: PixelArtSettings) -> Self {
    Self {
      pixel_lines: settings.pixel_lines,
      tone_bands: settings.tone_bands,
      hue_levels: settings.hue_levels,
      outline: settings.outline,
    }
  }
}

impl Default for PixelArt {
  fn default() -> Self {
    PixelArtSettings::default().into()
  }
}

impl FullscreenMaterial for PixelArt {
  fn fragment_shader() -> ShaderRef {
    embedded_shader(embedded_path!("pixel_art.wesl"))
  }

  fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
    schedule_in_style_pass(system)
  }
}
