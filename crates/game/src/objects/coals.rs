use super::{fire::Blaze, scatter, update_materials};
use bevy::{
  light::NotShadowCaster,
  pbr::{Material, MaterialPlugin},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};
use std::f32::consts::TAU;

const COAL_COUNT: u32 = 150;

pub fn coals_plugin(app: &mut App) {
  app
    .add_plugins(MaterialPlugin::<CoalMaterial>::default())
    .add_systems(Update, update_coals.run_if(resource_changed::<Blaze>));
}

#[derive(ShaderType, Clone, Copy, Debug)]
struct GpuBlaze {
  flare: f32,
  // webgl2 padding my old friend
  _webgl2_padding: Vec3,
}

impl From<Blaze> for GpuBlaze {
  fn from(blaze: Blaze) -> Self {
    Self {
      flare: blaze.flare,
      _webgl2_padding: Vec3::ZERO,
    }
  }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct CoalMaterial {
  #[uniform(0)]
  blaze: GpuBlaze,
}

impl Material for CoalMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/coals.wesl".into()
  }

  fn enable_shadows() -> bool {
    false
  }
}

pub fn spawn(
  blaze: Res<Blaze>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<CoalMaterial>>,
) -> Result {
  let lump = Sphere::new(1.0)
    .mesh()
    .ico(0)?
    .with_duplicated_vertices()
    .with_computed_flat_normals();
  let coal = |x: u32| {
    let spot = scatter(x, COAL_COUNT, 0.0, 0.98);
    let size = 0.055.lerp(0.115, spot.phase);
    lump.clone().transformed_by(Transform {
      translation: Vec3::new(spot.position.x, size * 0.45, spot.position.y),
      rotation: Quat::from_euler(EulerRot::XYZ, spot.phase * TAU, spot.pace * TAU, 0.0),
      scale: Vec3::new(size * 1.25, size * 0.75, size),
    })
  };
  let mut bed = coal(0);
  for x in 1..COAL_COUNT {
    bed.merge(&coal(x))?;
  }
  commands.spawn((
    Name::new("Coals"),
    Mesh3d(meshes.add(bed)),
    MeshMaterial3d(materials.add(CoalMaterial {
      blaze: (*blaze).into(),
    })),
    NotShadowCaster,
  ));
  Ok(())
}

fn update_coals(
  blaze: Res<Blaze>,
  coals: Query<&MeshMaterial3d<CoalMaterial>>,
  mut materials: ResMut<Assets<CoalMaterial>>,
) {
  update_materials(coals, &mut materials, |x| {
    x.blaze = (*blaze).into();
  });
}
