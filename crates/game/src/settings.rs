use crate::{
  conductor::ConductorSettings,
  gyms::main::MenuSettings,
  objects::{FireSettings, SkySettings, SnowSettings},
  ron_asset::{RonAsset, ron_asset_plugin},
};
use bevy::prelude::*;
use serde::Deserialize;
use std::mem::discriminant;
use sys_cam::PlayerCamera;
use sys_style::ScreenStyle;

pub fn settings_plugin(app: &mut App) {
  app
    .add_plugins(ron_asset_plugin::<GameSettings>)
    .add_systems(Startup, setup)
    .add_systems(Update, (apply_settings, cycle_style));
}

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
struct GameSettings {
  screen_style: ScreenStyle,
  sky: SkySettings,
  fire: FireSettings,
  snow: SnowSettings,
  menu: MenuSettings,
  conductor: ConductorSettings,
}

impl RonAsset for GameSettings {
  const EXTENSIONS: &'static [&'static str] = &["settings.ron"];
}

#[derive(Resource)]
struct SettingsFile(Handle<GameSettings>);

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
  let file = asset_server.load("settings/game.settings.ron");
  commands.insert_resource(SettingsFile(file));
}

fn apply_settings(
  mut events: MessageReader<AssetEvent<GameSettings>>,
  file: Res<SettingsFile>,
  files: Res<Assets<GameSettings>>,
  cameras: Query<Entity, With<PlayerCamera>>,
  mut commands: Commands,
) {
  let loaded = events
    .read()
    .any(|x| x.is_loaded_with_dependencies(&file.0) || x.is_modified(&file.0));
  let Some(settings) = files.get(&file.0).filter(|_| loaded) else {
    return;
  };
  commands.insert_resource(settings.sky);
  commands.insert_resource(settings.fire);
  commands.insert_resource(settings.snow);
  commands.insert_resource(settings.menu);
  commands.insert_resource(settings.conductor.clone());
  for camera in &cameras {
    commands.entity(camera).insert(settings.screen_style);
  }
}

fn cycle_style(
  keys: Res<ButtonInput<KeyCode>>,
  file: Res<SettingsFile>,
  files: Res<Assets<GameSettings>>,
  mut cameras: Query<&mut ScreenStyle, With<PlayerCamera>>,
) {
  if !keys.just_pressed(KeyCode::F2) {
    return;
  }
  let Some(settings) = files.get(&file.0) else {
    return;
  };
  for mut style in &mut cameras {
    let next = style.next();
    *style = if discriminant(&next) == discriminant(&settings.screen_style) {
      settings.screen_style
    } else {
      next
    };
  }
}
