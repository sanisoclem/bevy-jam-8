use super::{
  LANES_PER_ROW, packed_rows,
  sky::{SkySettings, Weather},
  update_materials,
};
use bevy::{
  pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};
use std::f32::consts::FRAC_PI_2;

pub const LEG_COUNT: usize = 3;
pub const BANDS_PER_LEG: usize = 48;

type GroundMaterial = ExtendedMaterial<StandardMaterial, GroundWarp>;

pub fn ground_plugin(app: &mut App) {
  app
    .add_plugins(MaterialPlugin::<GroundMaterial>::default())
    .init_resource::<ShadowWaves>()
    .add_systems(
      Update,
      update_ground.run_if(resource_changed::<ShadowWaves>),
    );
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ShadowWaves {
  pub levels: [[f32; BANDS_PER_LEG]; LEG_COUNT],
  pub band_count: usize,
  pub strike_level: f32,
  pub limit_scale: f32,
}

impl Default for ShadowWaves {
  fn default() -> Self {
    Self {
      levels: [[0.0; BANDS_PER_LEG]; LEG_COUNT],
      band_count: 0,
      strike_level: 0.0,
      limit_scale: 1.0,
    }
  }
}

#[derive(ShaderType, Clone, Copy, Debug)]
struct GpuShadowWaves {
  levels: [Vec4; LEG_COUNT * BANDS_PER_LEG / LANES_PER_ROW],
  band_count: f32,
  strike_level: f32,
  limit_scale: f32,
  // webgl2 padding my old friend
  _webgl2_padding: f32,
}

impl From<ShadowWaves> for GpuShadowWaves {
  fn from(waves: ShadowWaves) -> Self {
    Self {
      levels: packed_rows(waves.levels.into_iter().flatten()),
      band_count: waves.band_count as f32,
      strike_level: waves.strike_level,
      limit_scale: waves.limit_scale,
      _webgl2_padding: 0.0,
    }
  }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct GroundWarp {
  #[uniform(100)]
  waves: GpuShadowWaves,
}

impl MaterialExtension for GroundWarp {
  fn fragment_shader() -> ShaderRef {
    "shaders/ground.wesl".into()
  }
}

pub fn make_ground(
  settings: Res<SkySettings>,
  waves: Res<ShadowWaves>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<GroundMaterial>>,
) {
  let radius = match settings.weather {
    Weather::Clear => 14.0,
    Weather::Storm => 90.0,
  };
  let disc = Circle::new(radius).mesh().resolution(128);
  commands.spawn((
    Name::new("Ground"),
    Mesh3d(meshes.add(disc)),
    MeshMaterial3d(materials.add(GroundMaterial {
      base: StandardMaterial {
        base_color: Color::srgb(0.36, 0.345, 0.355),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
      },
      extension: GroundWarp {
        waves: (*waves).into(),
      },
    })),
    Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
  ));
}

fn update_ground(
  waves: Res<ShadowWaves>,
  grounds: Query<&MeshMaterial3d<GroundMaterial>>,
  mut materials: ResMut<Assets<GroundMaterial>>,
) {
  update_materials(grounds, &mut materials, |x| {
    x.extension.waves = (*waves).into();
  });
}
