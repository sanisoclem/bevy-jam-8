use super::{VisualEffect, billboard_quads, update_materials};
use bevy::{
  camera::visibility::NoFrustumCulling,
  light::NotShadowCaster,
  pbr::{Material, MaterialPlugin},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};

const GLOW_SHADER: &str = "shaders/pot_glow.wesl";

pub fn pot_glow_plugin(app: &mut App) {
  app
    .add_plugins(MaterialPlugin::<GlowMaterial>::default())
    .init_resource::<PotGlow>()
    .add_systems(Update, update_glow);
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct PotGlow {
  pub target: f32,
  pub strength: f32,
}

#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct GpuGlow {
  strength: f32,
  _webgl2_padding: Vec3,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct GlowMaterial {
  #[uniform(0)]
  glow: GpuGlow,
}

impl Material for GlowMaterial {
  fn vertex_shader() -> ShaderRef {
    GLOW_SHADER.into()
  }

  fn fragment_shader() -> ShaderRef {
    GLOW_SHADER.into()
  }

  fn alpha_mode(&self) -> AlphaMode {
    AlphaMode::Add
  }

  fn enable_shadows() -> bool {
    false
  }
}

pub fn spawner(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<GlowMaterial>>,
) {
  commands.spawn((
    Name::new("Pot Glow"),
    VisualEffect,
    Mesh3d(meshes.add(billboard_quads(1, |_| [0.0; 3], |_| [0.0; 2]))),
    MeshMaterial3d(materials.add(GlowMaterial {
      glow: GpuGlow::default(),
    })),
    Transform::from_xyz(0.0, 1.0, 0.0),
    NoFrustumCulling,
    NotShadowCaster,
  ));
}

fn update_glow(
  mut pot_glow: ResMut<PotGlow>,
  time: Res<Time>,
  glows: Query<&MeshMaterial3d<GlowMaterial>>,
  mut materials: ResMut<Assets<GlowMaterial>>,
) {
  let catch_up = 1.0 - (-time.delta_secs() / 4.0).exp();
  pot_glow.strength += (pot_glow.target - pot_glow.strength) * catch_up;
  update_materials(glows, &mut materials, |x| {
    x.glow.strength = pot_glow.strength;
  });
}
