use bevy::prelude::*;

pub mod card_edge;
pub mod look_around;
pub mod main;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gym {
  #[default]
  Main,
  LookAround,
  CardEdge,
}

impl Plugin for Gym {
  fn build(&self, app: &mut App) {
    match self {
      Gym::Main => app.add_plugins(main::MainGymPlugin),
      Gym::LookAround => app.add_plugins(look_around::LookAroundGymPlugin),
      Gym::CardEdge => app.add_plugins(card_edge::CardEdgeGymPlugin),
    };
  }
}
