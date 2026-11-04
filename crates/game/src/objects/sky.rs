use super::update_materials;
use bevy::{
  light::GlobalAmbientLight,
  pbr::{DistanceFog, Material, MaterialPlugin},
  prelude::*,
  render::render_resource::{AsBindGroup, ShaderType},
  shader::ShaderRef,
};
use serde::Deserialize;
use sys_cam::PlayerCamera;

const GOLDEN_RATIO: f32 = 1.618_034;

pub fn sky_plugin(app: &mut App) {
  app
    .add_plugins(MaterialPlugin::<SkyMaterial>::default())
    .init_resource::<SkyMotion>()
    .add_systems(Update, update_sky.run_if(resource_exists::<SkySettings>));
}

#[derive(Component, Debug, Default)]
pub struct WorldLight;

#[derive(Component, Debug, Default)]
struct Skylight;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weather {
  Clear,
  Storm,
}

impl Weather {
  fn storm(self) -> f32 {
    match self {
      Weather::Clear => 0.0,
      Weather::Storm => 1.0,
    }
  }

  fn haze(self) -> Color {
    match self {
      Weather::Clear => Color::srgb(0.04, 0.047, 0.07),
      Weather::Storm => Color::srgb(0.012, 0.014, 0.02),
    }
  }
}

#[derive(Resource, Deserialize, Clone, Copy, Debug)]
pub struct SkySettings {
  pub weather: Weather,
  pub star_lines: f32,
  pub haze_visibility: f32,
  pub lightning_brightness: f32,
}

impl SkySettings {
  fn skylight_brightness(&self, motion: SkyMotion) -> f32 {
    match self.weather {
      Weather::Clear => {
        let strike = (-motion.since_strike / 0.35).exp();
        1200.0 * (1.0 + 0.8 * strike * motion.activity)
      }
      Weather::Storm => 260.0 + self.lightning_brightness * motion.flash,
    }
  }
}

#[derive(Resource, ShaderType, Clone, Copy, Debug, Default)]
pub struct SkyMotion {
  pub since_strike: f32,
  pub strike_number: u32,
  pub flash: f32,
  pub intensity: f32,
  pub activity: f32,
  pub star_shine: f32,
  pub star_tick: f32,
}

#[derive(Default)]
struct BoltAim {
  strike_number: Option<u32>,
  heading: f32,
}

#[derive(ShaderType, Clone, Copy, Debug)]
struct Sky {
  haze: LinearRgba,
  star_lines: f32,
  storm: f32,
  bolt_heading: f32,
  #[shader(align(16))]
  motion: SkyMotion,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct SkyMaterial {
  #[uniform(0)]
  sky: Sky,
}

impl Material for SkyMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/sky.wesl".into()
  }
}

pub fn spawn(
  settings: Res<SkySettings>,
  camera: Single<Entity, With<PlayerCamera>>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<SkyMaterial>>,
) -> Result {
  let dome = Mesh::from(Sphere::new(400.0)).with_inverted_winding()?;
  let material = SkyMaterial {
    sky: Sky {
      haze: settings.weather.haze().into(),
      star_lines: settings.star_lines,
      storm: settings.weather.storm(),
      bolt_heading: 0.0,
      motion: SkyMotion::default(),
    },
  };
  let skylight = (
    Name::new("Skylight"),
    WorldLight,
    Skylight,
    DirectionalLight {
      color: Color::srgb(0.78, 0.82, 1.0),
      illuminance: settings.skylight_brightness(SkyMotion::default()),
      ..default()
    },
    Transform::from_xyz(-0.62, 0.4, -0.6).looking_at(Vec3::ZERO, Vec3::Y),
  );
  commands.spawn((
    Name::new("Sky"),
    Mesh3d(meshes.add(dome)),
    MeshMaterial3d(materials.add(material)),
    children![skylight],
  ));
  commands.insert_resource(GlobalAmbientLight {
    color: Color::srgb(0.62, 0.66, 0.8),
    brightness: 170.0,
    ..default()
  });
  commands.entity(*camera).insert(DistanceFog {
    color: settings.weather.haze(),
    falloff: FogFalloff::from_visibility(settings.haze_visibility),
    ..default()
  });
  Ok(())
}

fn update_sky(
  settings: Res<SkySettings>,
  motion: Res<SkyMotion>,
  camera: Single<&GlobalTransform, With<PlayerCamera>>,
  skies: Query<&MeshMaterial3d<SkyMaterial>>,
  skylights: Query<&mut DirectionalLight, With<Skylight>>,
  mut materials: ResMut<Assets<SkyMaterial>>,
  mut bolt: Local<BoltAim>,
) {
  if !settings.is_changed() && !motion.is_changed() {
    return;
  }
  if bolt.strike_number != Some(motion.strike_number) {
    bolt.strike_number = Some(motion.strike_number);
    let forward = camera.forward();
    let scatter = (motion.strike_number as f32 * GOLDEN_RATIO).fract() - 0.5;
    bolt.heading = forward.x.atan2(-forward.z) + scatter;
  }
  update_materials(skies, &mut materials, |x| {
    x.sky.star_lines = settings.star_lines;
    x.sky.storm = settings.weather.storm();
    x.sky.motion = *motion;
    x.sky.bolt_heading = bolt.heading;
  });
  for mut skylight in skylights {
    skylight.illuminance = settings.skylight_brightness(*motion);
  }
}
