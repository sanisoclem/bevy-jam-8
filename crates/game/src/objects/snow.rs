use super::{
  LANES_PER_ROW, VisualEffect, billboard_quads, packed_rows, random_share,
  sky::{SkySettings, Weather},
  update_materials,
};
use bevy::{
  camera::visibility::NoFrustumCulling,
  light::NotShadowCaster,
  pbr::{Material, MaterialPlugin},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};
use serde::Deserialize;

const SNOW_SHADER: &str = "shaders/snow.wesl";

pub const FORECAST_SAMPLES: usize = 48;
pub const FORECAST_STEP_SECS: f32 = 0.25;

pub fn snow_plugin(app: &mut App) {
  app
    .add_plugins(MaterialPlugin::<SnowMaterial>::default())
    .init_resource::<Snowfall>()
    .add_systems(Update, update_snow.run_if(resource_changed::<Snowfall>));
}

#[derive(Resource, Deserialize, Clone, Copy, Debug)]
pub struct SnowSettings {
  flake_count: u32,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct Snowfall {
  pub strength: [f32; FORECAST_SAMPLES],
  pub first_sample_age_secs: f32,
  pub flash: f32,
}

impl Default for Snowfall {
  fn default() -> Self {
    Self {
      strength: [0.5; FORECAST_SAMPLES],
      first_sample_age_secs: 0.0,
      flash: 0.0,
    }
  }
}

#[derive(ShaderType, Clone, Copy, Debug)]
struct GpuSnowfall {
  strength: [Vec4; FORECAST_SAMPLES / LANES_PER_ROW],
  first_sample_age: f32,
  sample_step: f32,
  flash: f32,
  // webgl2 padding my old friend
  _webgl2_padding: f32,
}

impl From<Snowfall> for GpuSnowfall {
  fn from(snowfall: Snowfall) -> Self {
    Self {
      strength: packed_rows(snowfall.strength),
      first_sample_age: snowfall.first_sample_age_secs,
      sample_step: FORECAST_STEP_SECS,
      flash: snowfall.flash,
      _webgl2_padding: 0.0,
    }
  }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SnowMaterial {
  #[uniform(0)]
  snowfall: GpuSnowfall,
}

impl Material for SnowMaterial {
  fn vertex_shader() -> ShaderRef {
    SNOW_SHADER.into()
  }

  fn fragment_shader() -> ShaderRef {
    SNOW_SHADER.into()
  }

  fn alpha_mode(&self) -> AlphaMode {
    AlphaMode::Add
  }

  fn enable_shadows() -> bool {
    false
  }
}

pub fn spawner(
  sky: Res<SkySettings>,
  settings: Res<SnowSettings>,
  snowfall: Res<Snowfall>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<SnowMaterial>>,
) {
  if sky.weather != Weather::Storm {
    return;
  }
  let flakes = billboard_quads(
    settings.flake_count,
    |x| [random_share(x, 1), random_share(x, 2), random_share(x, 3)],
    |x| [random_share(x, 4), random_share(x, 5)],
  );
  commands.spawn((
    Name::new("Snow"),
    VisualEffect,
    Mesh3d(meshes.add(flakes)),
    MeshMaterial3d(materials.add(SnowMaterial {
      snowfall: (*snowfall).into(),
    })),
    NoFrustumCulling,
    NotShadowCaster,
  ));
}

fn update_snow(
  snowfall: Res<Snowfall>,
  snows: Query<&MeshMaterial3d<SnowMaterial>>,
  mut materials: ResMut<Assets<SnowMaterial>>,
) {
  update_materials(snows, &mut materials, |x| {
    x.snowfall = (*snowfall).into();
  });
}
