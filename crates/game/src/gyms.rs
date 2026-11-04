use bevy::prelude::*;

pub mod look_around;
pub mod main;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gym {
  #[default]
  Main,
  LookAround,
}

impl Plugin for Gym {
  fn build(&self, app: &mut App) {
    match self {
      Gym::Main => app.add_plugins(main::MainGymPlugin),
      Gym::LookAround => app.add_plugins(look_around::LookAroundGymPlugin),
    };
  }
}
