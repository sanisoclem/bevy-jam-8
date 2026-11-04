use bevy::{
  asset::RenderAssetUsages,
  mesh::{Indices, PrimitiveTopology},
  prelude::*,
};
use sys_asset::SysAssetPlugin;

mod cauldron;
mod coals;
mod fire;
mod ground;
mod pot_glow;
mod sky;
mod snow;

pub use cauldron::CauldronAssets;
pub use fire::{Blaze, FireSettings};
pub use ground::{BANDS_PER_LEG, LEG_COUNT, ShadowWaves};
pub use pot_glow::PotGlow;
pub use sky::{SkyMotion, SkySettings, WorldLight};
pub use snow::{FORECAST_SAMPLES, FORECAST_STEP_SECS, SnowSettings, Snowfall};

const GOLDEN_ANGLE: f32 = 2.399_963;
const PHASE_STEP: f32 = 0.754_877_7;
const PACE_STEP: f32 = 0.569_840_3;
const LANES_PER_ROW: usize = 4;

// should be self contained
pub struct ObjectsPlugin;

#[derive(Component, Debug, Default)]
pub struct VisualEffect;

impl Plugin for ObjectsPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins((
      SysAssetPlugin::<CauldronAssets>::default(),
      coals::coals_plugin,
      fire::fire_plugin,
      ground::ground_plugin,
      pot_glow::pot_glow_plugin,
      sky::sky_plugin,
      snow::snow_plugin,
    ));
  }
}

pub fn spawner(world: &mut World) -> Result {
  world.run_system_cached::<(), _, _>(sky::spawn)?;
  world.run_system_cached(ground::make_ground)?;
  world.run_system_cached(cauldron::spawner)?;
  world.run_system_cached::<(), _, _>(coals::spawn)?;
  world.run_system_cached(fire::spawner)?;
  world.run_system_cached(pot_glow::spawner)?;
  world.run_system_cached(snow::spawner)?;
  Ok(())
}

fn billboard_quads(
  count: u32,
  origin: impl Fn(u32) -> [f32; 3],
  seed: impl Fn(u32) -> [f32; 2],
) -> Mesh {
  let quads = 0..count;
  let origins = quads.clone().flat_map(|x| [origin(x); 4]);
  let corners = quads
    .clone()
    .flat_map(|_| [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]);
  let seeds = quads.clone().flat_map(|x| [seed(x); 4]);
  let indices = quads.flat_map(|x| {
    let first = x * 4;
    [first, first + 1, first + 2, first, first + 2, first + 3]
  });

  Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::RENDER_WORLD,
  )
  .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, origins.collect::<Vec<_>>())
  .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, corners.collect::<Vec<_>>())
  .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, seeds.collect::<Vec<_>>())
  .with_inserted_indices(Indices::U32(indices.collect()))
}

fn packed_rows<const ROWS: usize>(values: impl IntoIterator<Item = f32>) -> [Vec4; ROWS] {
  let mut rows = [Vec4::ZERO; ROWS];
  for (index, value) in values.into_iter().take(ROWS * LANES_PER_ROW).enumerate() {
    rows[index / LANES_PER_ROW][index % LANES_PER_ROW] = value;
  }
  rows
}

fn update_materials<M: Material>(
  handles: Query<&MeshMaterial3d<M>>,
  materials: &mut Assets<M>,
  update: impl Fn(&mut M),
) {
  for handle in handles {
    if let Some(mut material) = materials.get_mut(&handle.0) {
      update(&mut material);
    }
  }
}

fn random_share(index: u32, salt: u32) -> f32 {
  let seed = index.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
  let state = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
  let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
  ((word >> 22) ^ word) as f32 / u32::MAX as f32
}

struct Spot {
  position: Vec2,
  phase: f32,
  pace: f32,
}

fn scatter(index: u32, count: u32, inner_radius: f32, outer_radius: f32) -> Spot {
  let step = index as f32;
  let spiral = ((step + 0.5) / count as f32).sqrt();
  let radius = inner_radius.lerp(outer_radius, spiral);
  let (sine, cosine) = (step * GOLDEN_ANGLE).sin_cos();
  Spot {
    position: Vec2::new(radius * cosine, radius * sine),
    phase: (step * PHASE_STEP).fract(),
    pace: (step * PACE_STEP).fract(),
  }
}
