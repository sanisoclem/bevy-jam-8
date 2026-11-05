use super::update_materials;
use bevy::{
  light::NotShadowCaster,
  pbr::{Material, MaterialPlugin},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};
use std::f32::consts::FRAC_PI_2;

pub fn fluid_plugin(app: &mut App) {
  app
    .add_plugins(MaterialPlugin::<FluidMaterial>::default())
    .init_resource::<CauldronFluid>()
    .add_systems(
      Update,
      update_fluid.run_if(resource_changed::<CauldronFluid>),
    );
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct CauldronFluid {
  pub color: Color,
  pub icon: Option<u32>,
}

impl Default for CauldronFluid {
  fn default() -> Self {
    Self {
      color: Color::srgb(0.2, 0.42, 0.25),
      icon: None,
    }
  }
}

#[derive(ShaderType, Clone, Copy, Debug)]
struct GpuFluid {
  color: LinearRgba,
  icon: f32,
  _webgl2_padding: Vec3,
}

impl From<CauldronFluid> for GpuFluid {
  fn from(fluid: CauldronFluid) -> Self {
    Self {
      color: fluid.color.into(),
      icon: fluid.icon.map_or(-1.0, |x| x as f32),
      _webgl2_padding: Vec3::ZERO,
    }
  }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct FluidMaterial {
  #[uniform(0)]
  fluid: GpuFluid,
  #[texture(1)]
  #[sampler(2)]
  sheet: Handle<Image>,
}

impl Material for FluidMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/fluid.wesl".into()
  }
}

pub fn spawn(
  fluid: Res<CauldronFluid>,
  asset_server: Res<AssetServer>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<FluidMaterial>>,
) {
  commands.spawn((
    Name::new("Fluid"),
    Mesh3d(meshes.add(Circle::new(0.7).mesh().resolution(64))),
    MeshMaterial3d(materials.add(FluidMaterial {
      fluid: (*fluid).into(),
      sheet: asset_server.load("cards/sheet.png"),
    })),
    Transform::from_xyz(0.0, 0.95, 0.0).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
    NotShadowCaster,
  ));
}

fn update_fluid(
  fluid: Res<CauldronFluid>,
  fluids: Query<&MeshMaterial3d<FluidMaterial>>,
  mut materials: ResMut<Assets<FluidMaterial>>,
) {
  update_materials(fluids, &mut materials, |x| {
    x.fluid = (*fluid).into();
  });
}
