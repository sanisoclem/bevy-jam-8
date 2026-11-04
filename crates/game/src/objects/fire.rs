use super::{WorldLight, billboard_quads, scatter};
use bevy::{
  camera::visibility::NoFrustumCulling,
  light::NotShadowCaster,
  pbr::{Material, MaterialPlugin},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};
use serde::Deserialize;

const EMBER_SHADER: &str = "shaders/embers.wesl";
const EMBER_COUNT: u32 = 64;
const FIRE_LIGHT_BRIGHTNESS: f32 = 240_000.0;

const CAULDRON_INFO: CauldronInfo = CauldronInfo {
  center_y: 0.68,
  half_height: 0.42,
  radius: 0.82,
  ember_ceiling: 0.86,
};

pub fn fire_plugin(app: &mut App) {
  app
    .add_plugins(MaterialPlugin::<EmberMaterial>::default())
    .init_resource::<Blaze>()
    .add_systems(Update, update_fire.run_if(resource_exists::<FireSettings>));
}

#[derive(Resource, Deserialize, Clone, Copy, Debug)]
pub struct FireSettings {
  flare_light: f32,
}

#[derive(ShaderType, Clone, Copy, Debug)]
struct CauldronInfo {
  center_y: f32,
  half_height: f32,
  radius: f32,
  ember_ceiling: f32,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Blaze {
  pub flare: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct EmberMaterial {
  #[uniform(0)]
  cauld: CauldronInfo,
}

impl Material for EmberMaterial {
  fn vertex_shader() -> ShaderRef {
    EMBER_SHADER.into()
  }

  fn fragment_shader() -> ShaderRef {
    EMBER_SHADER.into()
  }

  fn alpha_mode(&self) -> AlphaMode {
    AlphaMode::Add
  }

  fn enable_shadows() -> bool {
    false
  }
}

#[derive(Component, Debug)]
struct FireLight;

pub fn spawner(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<EmberMaterial>>,
) {
  let spot = |x| scatter(x, EMBER_COUNT, 0.25, 0.95);
  let quads = billboard_quads(
    EMBER_COUNT,
    |x| [spot(x).position.x, 0.08, spot(x).position.y],
    |x| [spot(x).phase, spot(x).pace],
  );
  let embers = (
    Name::new("Embers"),
    Mesh3d(meshes.add(quads)),
    MeshMaterial3d(materials.add(EmberMaterial {
      cauld: CAULDRON_INFO,
    })),
    NoFrustumCulling,
    NotShadowCaster,
  );
  let light = (
    Name::new("Fire Light"),
    WorldLight,
    FireLight,
    PointLight {
      color: Color::srgb(1.0, 0.27, 0.07),
      intensity: FIRE_LIGHT_BRIGHTNESS,
      range: 14.0,
      radius: 0.3,
      shadow_maps_enabled: true,
      ..default()
    },
    Transform::from_xyz(0.0, 0.2, 0.0),
  );
  commands.spawn((
    Name::new("Fire"),
    Transform::default(),
    Visibility::default(),
    children![embers, light],
  ));
}

fn update_fire(
  lights: Query<&mut PointLight, With<FireLight>>,
  settings: Res<FireSettings>,
  blaze: Res<Blaze>,
  time: Res<Time>,
) {
  let secs = time.elapsed_secs();
  let wave = (secs * 0.7).sin() * 0.6 + (secs * 1.13 + 1.3).sin() * 0.3 + (secs * 0.31).sin() * 0.1;
  let flare = 1.0 + settings.flare_light * blaze.flare;
  for mut light in lights {
    light.intensity = FIRE_LIGHT_BRIGHTNESS * (0.72 + wave * 0.28) * flare;
  }
}
