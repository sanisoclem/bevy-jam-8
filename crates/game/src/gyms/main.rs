use bevy::prelude::*;

pub struct MainGymPlugin;

impl Plugin for MainGymPlugin {
  fn build(&self, app: &mut App) {
    app.init_state::<GameState>().add_plugins((
      splash::splash_plugin,
      menu::menu_plugin,
      game::game_plugin,
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
  use sys_asset::{AssetLoaded, LoadAssets};
  use sys_audio::GameAudioAssets;

  use super::GameState;

  pub fn splash_plugin(app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Splash), splash_setup)
      .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
      .add_observer(on_audio_loaded);
  }

  #[derive(Component)]
  struct OnSplashScreen {
    audio_loaded: bool,
    timer: Timer,
  }

  fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon = asset_server.load("branding/icon.png");

    commands.trigger(LoadAssets::<GameAudioAssets>::default());
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
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
  ) {
    for mut splash in qry {
      if !splash.timer.is_finished() {
        splash.timer.tick(time.delta());
      }

      if splash.timer.is_finished() && splash.audio_loaded {
        game_state.set(GameState::Menu);
      }
    }
  }

  fn on_audio_loaded(_: On<AssetLoaded<GameAudioAssets>>, qry: Query<&mut OnSplashScreen>) {
    for mut splash in qry {
      splash.audio_loaded = true;
    }
  }
}

mod menu {
  use bevy::{
    app::AppExit,
    color::palettes::css::WHITE,
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{Activate, Button},
  };
  use sys_audio::{GameAudioChannels, GameAudioCommand, GameAudioLibrary};

  use super::{GameState, TEXT_COLOR};

  pub fn menu_plugin(app: &mut App) {
    app
      .add_systems(OnEnter(GameState::Menu), (main_menu_setup, play_menu_music))
      .add_systems(
        Update,
        update_button_colors.run_if(in_state(GameState::Menu)),
      )
      .add_observer(on_menu_button_activated);
  }

  #[derive(Component)]
  struct OnMainMenuScreen;

  const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
  const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
  const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

  #[derive(Component)]
  enum MenuButtonAction {
    Play,
    Quit,
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

  fn play_menu_music(mut commands: Commands) {
    commands.trigger(GameAudioCommand::ReplaceAllAndFadeInto(
      GameAudioLibrary::Menu,
      GameAudioChannels::Music,
    ));
  }

  fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_node = Node {
      width: px(300),
      height: px(65),
      margin: UiRect::all(px(20)),
      justify_content: JustifyContent::Center,
      align_items: AlignItems::Center,
      ..default()
    };
    let button_text_font = TextFont {
      font_size: FontSize::Px(33.0),
      ..default()
    };

    let texture_handle = asset_server.load("ui/bg.png");
    commands.spawn((
      DespawnOnExit(GameState::Menu),
      Node {
        width: percent(100),
        height: percent(100),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
      },
      BackgroundColor(WHITE.into()),
      ImageNode::from_atlas_image(texture_handle.clone(), TextureAtlas::default()),
      OnMainMenuScreen,
      children![(
        Node {
          flex_direction: FlexDirection::Column,
          align_items: AlignItems::Center,
          margin: UiRect::top(Val::Px(200.)),
          ..default()
        },
        children![
          (
            Text::new(""),
            TextFont {
              font_size: FontSize::Px(90.0),
              ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
              margin: UiRect::all(px(50)),
              ..default()
            },
          ),
          (
            Button,
            Hovered::default(),
            button_node.clone(),
            BackgroundColor(NORMAL_BUTTON),
            MenuButtonAction::Play,
            children![(
              Text::new("Play"),
              button_text_font.clone(),
              TextColor(TEXT_COLOR),
            ),]
          ),
          (
            Button,
            Hovered::default(),
            button_node,
            BackgroundColor(NORMAL_BUTTON),
            MenuButtonAction::Quit,
            children![(Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),]
          ),
        ]
      )],
    ));
  }

  fn on_menu_button_activated(
    activate: On<Activate>,
    actions: Query<&MenuButtonAction>,
    mut commands: Commands,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
  ) {
    let Ok(action) = actions.get(activate.entity) else {
      return;
    };
    commands.trigger(GameAudioCommand::InsertOnce(
      GameAudioLibrary::ButtonEffect,
      GameAudioChannels::UI,
    ));
    match action {
      MenuButtonAction::Quit => {
        app_exit_writer.write(AppExit::Success);
      }
      MenuButtonAction::Play => {
        game_state.set(GameState::Playing);
      }
    }
  }
}

mod game {
  use bevy::prelude::*;
  use sys_audio::{GameAudioChannels, GameAudioCommand, GameAudioLibrary};

  use super::GameState;

  pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), game_setup);
  }

  fn game_setup(mut commands: Commands) {
    commands.spawn((
      Name::new("Game Scene"),
      DespawnOnExit(GameState::Playing),
      Transform::default(),
      Visibility::default(),
    ));
    commands.trigger(GameAudioCommand::ReplaceAllAndFadeInto(
      GameAudioLibrary::Lofi,
      GameAudioChannels::Music,
    ));
  }
}
