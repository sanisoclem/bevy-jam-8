use crate::{
  conductor::ConductorPlugin,
  gyms::Gym,
  objects::{ObjectsPlugin, VisualEffect, WorldLight},
  settings::settings_plugin,
  table::Card,
};
use bevy::{
  asset::AssetMetaCheck,
  audio::{AudioPlugin, SpatialScale},
  prelude::*,
};
use jam8::prelude::*;
use sys_anim::SysAnimPlugin;
use sys_audio::SysAudioPlugin;
use sys_cam::{PlayerCamera, SysCamPlugin};
use sys_style::{Lens, SysStylePlugin, Unstyled, UnstyledLight};

pub struct GamePlugin {
  pub gym: Gym,
}

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(
        DefaultPlugins
          .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
          })
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
      .add_plugins((
        Jam8Plugin,
        SysAnimPlugin,
        SysAudioPlugin,
        SysStylePlugin,
        SysCamPlugin,
        ObjectsPlugin,
        ConductorPlugin,
        settings_plugin,
        self.gym,
      ))
      .add_observer(on_player_camera_added)
      .add_observer(on_world_light_added)
      .add_observer(on_visual_effect_added)
      .add_observer(on_card_added);

    #[cfg(feature = "dev")]
    app.add_plugins(dev::DevGamePlugin);
  }
}

fn on_player_camera_added(added: On<Add<PlayerCamera>>, mut commands: Commands) {
  commands.entity(added.entity).insert(Lens::default());
}

fn on_world_light_added(added: On<Add<WorldLight>>, mut commands: Commands) {
  commands.entity(added.entity).insert(UnstyledLight);
}

fn on_visual_effect_added(added: On<Add<VisualEffect>>, mut commands: Commands) {
  commands.entity(added.entity).insert(Unstyled);
}

fn on_card_added(added: On<Add<Card>>, mut commands: Commands) {
  commands.entity(added.entity).insert(Unstyled);
}

mod caption;
mod conductor;
#[cfg(feature = "dev")]
pub mod dev;
mod director;
pub mod gyms;
pub mod objects;
mod ron_asset;
mod settings;
mod table;
pub mod prelude {
  pub use crate::GamePlugin;
}
