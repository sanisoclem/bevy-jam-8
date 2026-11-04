use crate::{embedded_shader, style_pass::schedule_after_style_pass};
use bevy::{
  asset::{embedded_asset, embedded_path},
  core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
  ecs::{schedule::ScheduleConfigs, system::BoxedSystem},
  prelude::*,
  render::{RenderApp, extract_component::ExtractComponent, render_resource::ShaderType},
  shader::ShaderRef,
};

pub fn lens_plugin(app: &mut App) {
  embedded_asset!(app, "lens.wesl");
  app.add_plugins(FullscreenMaterialPlugin::<Lens>::default());
}

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Debug, Default)]
#[extract_app(RenderApp)]
pub struct Lens {
  pub blur: f32,
  pub shake: Vec2,
}

impl FullscreenMaterial for Lens {
  fn fragment_shader() -> ShaderRef {
    embedded_shader(embedded_path!("lens.wesl"))
  }

  fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
    schedule_after_style_pass(system)
  }
}
