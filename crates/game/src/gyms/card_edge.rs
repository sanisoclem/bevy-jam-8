use crate::{
  objects::WorldLight,
  table::{self, CardFace, CardMaker},
};
use bevy::prelude::*;
use sys_cam::{CameraPose, LookAroundCamera};

pub struct CardEdgeGymPlugin;

impl Plugin for CardEdgeGymPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(table::table_plugin)
      .add_systems(PostStartup, setup)
      .add_systems(Update, spin);
  }
}

#[derive(Component, Debug)]
struct SpinningCard {
  axis: Vec3,
}

fn setup(mut maker: CardMaker) {
  let eye = Vec3::new(0.0, 0.0, 3.2);
  maker.commands.trigger(LookAroundCamera(CameraPose {
    eye,
    target: Vec3::ZERO,
  }));
  maker.commands.spawn((
    WorldLight,
    PointLight {
      intensity: 250_000.0,
      range: 20.0,
      ..default()
    },
    Transform::from_xyz(1.5, 1.5, 2.0),
  ));

  let shades = [
    Color::srgb(0.925, 0.231, 0.188),
    Color::srgb(0.106, 0.424, 0.122),
    Color::srgb(1.0, 0.835, 0.18),
  ];
  let face = |x: usize| CardFace {
    shade: shades[x % shades.len()],
    icon: (x % 6) as u32,
  };
  for (y, axis) in [(0.65, Vec3::NEG_X), (0.0, Vec3::Y)] {
    for column in 0..6 {
      let place = Vec3::new(column as f32 * 0.5 - 1.25, y, 0.0);
      let facing = Transform::from_translation(place).looking_to(place - eye, Vec3::Y);
      let angle = (column as f32 * 15.0).to_radians();
      maker.spawn(face(column)).insert((
        Transform {
          rotation: facing.rotation * Quat::from_axis_angle(axis, angle),
          scale: Vec3::splat(0.5),
          ..facing
        },
        Visibility::Visible,
      ));
    }
  }
  for (column, axis) in [Vec3::X, Vec3::Y, Vec3::new(1.0, 1.0, 0.0).normalize()]
    .into_iter()
    .enumerate()
  {
    maker.spawn(face(column + 3)).insert((
      SpinningCard { axis },
      Transform::from_xyz(column as f32 * 0.5 - 0.5, -0.65, 0.0).with_scale(Vec3::splat(0.5)),
      Visibility::Visible,
    ));
  }
}

fn spin(time: Res<Time>, cards: Query<(&SpinningCard, &mut Transform)>) {
  for (card, mut transform) in cards {
    transform.rotation = Quat::from_axis_angle(card.axis, time.elapsed_secs() * 0.5);
  }
}
