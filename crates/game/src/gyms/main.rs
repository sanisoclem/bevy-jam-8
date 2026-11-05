use bevy::prelude::*;
use serde::Deserialize;
use sys_cam::CameraPose;

#[derive(Resource, Deserialize, Clone, Copy, Debug)]
pub struct MenuSettings {
  menu_pose: CameraPose,
  cauldron_pose: CameraPose,
  orbit_rate: f32,
  orbit_start_degrees: f32,
  music_delay_secs: f32,
  music_phrase_secs: f32,
  music_crossfade_secs: f32,
}

pub struct MainGymPlugin;

impl Plugin for MainGymPlugin {
  fn build(&self, app: &mut App) {
    app.init_state::<GameState>().add_plugins((
      splash::splash_plugin,
      menu::menu_plugin,
      game::game_plugin,
      crate::table::table_plugin,
      crate::caption::caption_plugin,
      crate::director::director_plugin,
    ));
  }
}

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GameState {
  #[default]
  Splash,
  Menu,
  Playing,
}

mod splash {
  use bevy::prelude::*;
  use jam8::prelude::GameBlueprint;
  use sys_asset::{AssetLoaded, LoadAssets};
  use sys_audio::GameAudioAssets;

  use super::{GameState, MenuSettings};
  use crate::objects::CauldronAssets;

  pub fn splash_plugin(app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Splash), splash_setup)
      .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
      .add_observer(on_audio_loaded)
      .add_observer(on_cauldron_loaded);
  }

  #[derive(Component)]
  struct OnSplashScreen {
    audio_loaded: bool,
    cauldron_loaded: bool,
    timer: Timer,
  }

  fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("branding/icon.png");

    commands.trigger(LoadAssets::<GameAudioAssets>::default());
    commands.trigger(LoadAssets::<CauldronAssets>::default());
    commands.spawn((
      DespawnOnExit(GameState::Splash),
      Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        width: percent(100),
        height: percent(100),
        ..default()
      },
      OnSplashScreen {
        timer: Timer::from_seconds(5.0, TimerMode::Once),
        audio_loaded: false,
        cauldron_loaded: false,
      },
      children![(
        ImageNode::new(icon),
        Node {
          width: px(200),
          ..default()
        },
      )],
    ));
  }

  fn countdown(
    qry: Query<&mut OnSplashScreen>,
    settings: Option<Res<MenuSettings>>,
    blueprint: Option<Res<GameBlueprint>>,
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
  ) {
    let files_loaded = settings.is_some() && blueprint.is_some();
    for mut splash in qry {
      if !splash.timer.is_finished() {
        splash.timer.tick(time.delta());
      }

      let assets_loaded = splash.audio_loaded && splash.cauldron_loaded && files_loaded;
      if splash.timer.is_finished() && assets_loaded {
        game_state.set(GameState::Menu);
      }
    }
  }

  fn on_audio_loaded(_: On<AssetLoaded<GameAudioAssets>>, qry: Query<&mut OnSplashScreen>) {
    for mut splash in qry {
      splash.audio_loaded = true;
    }
  }

  fn on_cauldron_loaded(_: On<AssetLoaded<CauldronAssets>>, qry: Query<&mut OnSplashScreen>) {
    for mut splash in qry {
      splash.cauldron_loaded = true;
    }
  }
}

mod menu {
  use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{Activate, Button},
  };
  use std::time::Duration;
  use sys_audio::{GameAudioChannels, GameAudioCommand, GameAudioLibrary, Speed};
  use sys_cam::OrbitCamera;

  use super::{GameState, MenuSettings, TEXT_COLOR};
  use crate::objects;

  pub fn menu_plugin(app: &mut App) {
    app
      .add_systems(
        OnEnter(GameState::Menu),
        (objects::spawner, main_menu_setup),
      )
      .add_systems(Update, update_button_colors)
      .add_systems(Update, play_menu_music.run_if(in_state(GameState::Menu)))
      .add_observer(on_menu_button_activated);
  }

  const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
  const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
  const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

  #[derive(Component)]
  enum MenuButtonAction {
    Play,
  }

  fn button_color(hovered: Hovered, pressed: bool) -> Color {
    match (pressed, hovered.get()) {
      (true, _) => PRESSED_BUTTON,
      (false, true) => HOVERED_BUTTON,
      (false, false) => NORMAL_BUTTON,
    }
  }

  fn update_button_colors(
    buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<Button>>,
  ) {
    for (hovered, pressed, mut background_color) in buttons {
      background_color.set_if_neq(BackgroundColor(button_color(*hovered, pressed)));
    }
  }

  fn play_menu_music(
    settings: Res<MenuSettings>,
    time: Res<Time>,
    mut waited: Local<Duration>,
    mut commands: Commands,
  ) {
    let delay = Duration::from_secs_f32(settings.music_delay_secs);
    let already_played = *waited >= delay;
    *waited += time.delta();
    if already_played || *waited < delay {
      return;
    }
    commands.trigger(GameAudioCommand::InsertOnce(
      GameAudioLibrary::Menu,
      GameAudioChannels::Music,
      Speed::Exact,
    ));
  }

  fn main_menu_setup(settings: Res<MenuSettings>, mut commands: Commands) {
    commands.trigger(OrbitCamera {
      pose: settings.menu_pose,
      radians_per_sec: settings.orbit_rate,
      start_angle: settings.orbit_start_degrees.to_radians(),
    });
    commands.spawn((
      DespawnOnExit(GameState::Menu),
      Node {
        width: percent(100),
        height: percent(100),
        align_items: AlignItems::FlexEnd,
        justify_content: JustifyContent::FlexEnd,
        ..default()
      },
      children![(
        Button,
        Hovered::default(),
        Node {
          width: px(300),
          height: px(65),
          margin: UiRect::all(px(40)),
          justify_content: JustifyContent::Center,
          align_items: AlignItems::Center,
          ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        MenuButtonAction::Play,
        children![(
          Text::new("Play"),
          TextFont {
            font_size: FontSize::Px(33.0),
            ..default()
          },
          TextColor(TEXT_COLOR),
        )],
      )],
    ));
  }

  fn on_menu_button_activated(
    activate: On<Activate>,
    actions: Query<&MenuButtonAction>,
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
  ) {
    let Ok(action) = actions.get(activate.entity) else {
      return;
    };
    commands.trigger(GameAudioCommand::InsertOnce(
      GameAudioLibrary::ButtonEffect,
      GameAudioChannels::UI,
      Speed::Varied,
    ));
    match action {
      MenuButtonAction::Play => {
        game_state.set(GameState::Playing);
      }
    }
  }
}

mod game {
  use bevy::prelude::*;
  use sys_audio::{Crossfade, GameAudioChannels, GameAudioCommand, GameAudioLibrary};
  use sys_cam::WalkCamera;

  use super::{GameState, MenuSettings};
  use crate::director::Director;

  pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), game_setup);
  }

  fn game_setup(settings: Res<MenuSettings>, mut commands: Commands) {
    commands.spawn((
      Name::new("Game Scene"),
      DespawnOnExit(GameState::Playing),
      Transform::default(),
      Visibility::default(),
    ));
    commands.spawn((
      Name::new("Director"),
      Director::default(),
      DespawnOnExit(GameState::Playing),
    ));
    commands.trigger(WalkCamera {
      to: settings.cauldron_pose,
      secs: 3.2,
    });
    commands.trigger(GameAudioCommand::ReplaceAllOnPhrase(
      GameAudioLibrary::Lofi,
      GameAudioChannels::Music,
      Crossfade {
        phrase_secs: settings.music_phrase_secs,
        fade_secs: settings.music_crossfade_secs,
      },
    ));
  }
}
