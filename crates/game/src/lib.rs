use crate::gyms::main::MainGymPlugin;
use bevy::{
  audio::{AudioPlugin, SpatialScale},
  prelude::*,
};
use jam8::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(
        DefaultPlugins
          .set(WindowPlugin {
            primary_window: Some(Window {
              canvas: Some("#main-canvas".into()),
              ..default()
            }),

            ..default()
          })
          .set(AudioPlugin {
            default_spatial_scale: SpatialScale::new_2d(1. / 100.),
            ..default()
          }),
      )
      .add_plugins(Jam8Plugin)
      .add_plugins(MainGymPlugin);

    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevGamePlugin);
  }
}

#[cfg(feature = "dev")]
pub mod dev;
pub mod gyms;
pub mod prelude {
  pub use crate::GamePlugin;
}
