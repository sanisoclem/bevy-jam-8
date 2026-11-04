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
use std::f32::consts::{PI, TAU};

pub fn hand_drawn_plugin(app: &mut App) {
  embedded_asset!(app, "hand_drawn.wesl");
  app
    .add_plugins(FullscreenMaterialPlugin::<HandDrawn>::default())
    .add_systems(PostUpdate, follow_camera.after(TransformSystems::Propagate));
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct HandDrawnSettings {
  pub hatch_lines: f32,
}

impl Default for HandDrawnSettings {
  fn default() -> Self {
    Self { hatch_lines: 180.0 }
  }
}

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Debug)]
#[extract_app(RenderApp)]
pub struct HandDrawn {
  hatch_lines: f32,
  focal_length: f32,
  unwrapped_heading: f32,
  pitch: f32,
}

impl From<HandDrawnSettings> for HandDrawn {
  fn from(settings: HandDrawnSettings) -> Self {
    Self {
      hatch_lines: settings.hatch_lines,
      focal_length: 1.0,
      unwrapped_heading: 0.0,
      pitch: 0.0,
    }
  }
}

impl Default for HandDrawn {
  fn default() -> Self {
    HandDrawnSettings::default().into()
  }
}

impl FullscreenMaterial for HandDrawn {
  fn fragment_shader() -> ShaderRef {
    embedded_shader(embedded_path!("hand_drawn.wesl"))
  }

  fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
    schedule_in_style_pass(system)
  }
}

fn follow_camera(drawings: Query<(&mut HandDrawn, &GlobalTransform, &Projection)>) {
  for (mut drawing, transform, projection) in drawings {
    let forward = transform.forward();
    drawing.unwrapped_heading += shortest_turn(drawing.unwrapped_heading, view_heading(forward));
    drawing.pitch = view_pitch(forward);
    if let Projection::Perspective(perspective) = projection {
      drawing.focal_length = focal_length(perspective.fov);
    }
  }
}

fn view_heading(forward: Dir3) -> f32 {
  forward.x.atan2(-forward.z)
}

fn view_pitch(forward: Dir3) -> f32 {
  forward.y.asin()
}

fn shortest_turn(from: f32, to: f32) -> f32 {
  (to - from + PI).rem_euclid(TAU) - PI
}

fn focal_length(fov: f32) -> f32 {
  0.5 / (fov * 0.5).tan()
}
