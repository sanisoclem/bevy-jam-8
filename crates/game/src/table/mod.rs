use crate::{objects::CauldronFluid, ron_asset::ron_asset_plugin};
use bevy::{
  picking::{events::PointerClick, hover::Hovered, mesh_picking::MeshPickingPlugin},
  prelude::*,
  ui_render::UiMaterialPlugin,
  ui_widgets::{Activate, Button},
};
use blueprint_file::{
  AlchemyPalette, BLUEPRINT_PATH, GameBlueprintFile, GameBlueprintHandle, load_blueprint,
};
use card::{CardAtlas, CardHudMaterial, CardMaterial, CardPose};
use jam8::prelude::*;
use sys_audio::{GameAudioChannels, GameAudioCommand, GameAudioLibrary, Speed};
use sys_cam::{CameraOrbit, CameraWalk, PlayerCamera};
use sys_style::Lens;

mod blueprint_file;
mod card;

pub use card::{Card, CardFace, CardMaker};

// TODO: move to game settings
const HAND_ROW: CardRow = CardRow {
  y: -0.37,
  size: 0.3,
  spacing: 0.22,
  lean: 0.25,
  fan: 0.06,
};
const OPPONENT_ROW: CardRow = CardRow {
  y: 0.42,
  size: 0.26,
  spacing: 0.22,
  lean: -0.15,
  fan: 0.0,
};

pub fn table_plugin(app: &mut App) {
  app
    .add_plugins((
      MeshPickingPlugin,
      MaterialPlugin::<CardMaterial>::default(),
      UiMaterialPlugin::<CardHudMaterial>::default(),
      ron_asset_plugin::<GameBlueprintFile>,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (load_blueprint, read_keys).before(SimSystems))
    .add_systems(
      Update,
      (
        display_cards.run_if(on_message::<RoundDealt>),
        update_cauldron.run_if(on_message::<CardPlayed>),
        update_cards,
      )
        .chain()
        .after(SimSystems),
    )
    .add_observer(on_table_button_activated);
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
struct CardSlot {
  hand: Entity,
  card: usize,
}

#[derive(Component, Debug)]
struct TableHud;

#[derive(Component, Debug)]
enum TableButtonAction {
  Pass,
  Reset,
}

struct CardRow {
  y: f32,
  size: f32,
  spacing: f32,
  lean: f32,
  fan: f32,
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
  commands.insert_resource(GameBlueprintHandle(asset_server.load(BLUEPRINT_PATH)));
  // hardcoded the card size in too many places...
  commands.insert_resource(CardAtlas {
    mesh: meshes.add(Rectangle::new(422.0 / 586.0, 1.0)),
    sheet: asset_server.load("cards/sheet.png"),
  });
}

fn item_face(palette: &AlchemyPalette, item: AlchemyItem) -> CardFace {
  CardFace {
    shade: palette.shade(item.color),
    icon: palette.icon(item.item_type),
  }
}

fn display_cards(
  mut dealt: MessageReader<RoundDealt>,
  games: Query<(&Game, &Round)>,
  player_hands: Query<&PlayerCardHand>,
  opponent_hands: Query<&OpponentCardHand>,
  palette: Res<AlchemyPalette>,
  camera: Single<Entity, With<PlayerCamera>>,
  shown: Query<Entity, Or<(With<Card>, With<TableHud>)>>,
  mut fluid: ResMut<CauldronFluid>,
  mut maker: CardMaker,
) {
  let Some(RoundDealt { game }) = dealt.read().last() else {
    return;
  };
  let Ok((
    Game {
      player, opponent, ..
    },
    round,
  )) = games.get(*game)
  else {
    return;
  };
  let (Ok(player_hand), Ok(opponent_hand)) =
    (player_hands.get(*player), opponent_hands.get(*opponent))
  else {
    return;
  };
  for entity in &shown {
    maker.commands.entity(entity).despawn();
  }
  *fluid = CauldronFluid::default();

  for (card, item) in player_hand.cards.iter().enumerate() {
    maker
      .spawn(item_face(&palette, *item))
      .insert((
        CardSlot {
          hand: *player,
          card,
        },
        ChildOf(*camera),
      ))
      .observe(on_card_clicked);
  }
  for (card, item) in opponent_hand.cards.iter().enumerate() {
    if let Some(item) = item {
      maker.spawn(item_face(&palette, *item)).insert((
        CardSlot {
          hand: *opponent,
          card,
        },
        ChildOf(*camera),
      ));
    }
  }

  let target = maker.hud(item_face(&palette, round.target));
  let label = |text: &str| {
    (
      Text::new(text),
      TextFont {
        font_size: FontSize::Vh(3.0),
        ..default()
      },
      TextColor(Color::srgb(0.9, 0.9, 0.9)),
    )
  };
  let button = |text: &str, action| {
    (
      Button,
      Hovered::default(),
      Node {
        width: Val::Vh(20.0),
        height: Val::Vh(6.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
      },
      BackgroundColor::default(),
      action,
      children![label(text)],
    )
  };
  maker.commands.spawn((
    TableHud,
    Node {
      width: percent(100),
      height: percent(100),
      ..default()
    },
    Pickable::IGNORE, // need to click through it
    Visibility::Hidden,
    children![
      (
        Node {
          position_type: PositionType::Absolute,
          top: Val::Vh(3.0),
          right: Val::Vh(3.0),
          width: Val::Vh(22.0 * 422.0 / 586.0),
          height: Val::Vh(22.0),
          ..default()
        },
        target,
      ),
      (
        Node {
          position_type: PositionType::Absolute,
          top: Val::Vh(26.0),
          right: Val::Vh(3.0),
          width: Val::Vh(22.0 * 422.0 / 586.0),
          justify_content: JustifyContent::Center,
          ..default()
        },
        children![label("Target")],
      ),
      (
        Node {
          position_type: PositionType::Absolute,
          right: Val::Vh(3.0),
          bottom: Val::Vh(36.0),
          flex_direction: FlexDirection::Column,
          row_gap: Val::Vh(1.5),
          ..default()
        },
        children![
          button("Pass", TableButtonAction::Pass),
          button("Reset", TableButtonAction::Reset),
        ],
      ),
    ],
  ));
}

fn update_cauldron(
  mut played: MessageReader<CardPlayed>,
  player_hands: Query<(), With<PlayerCardHand>>,
  palette: Res<AlchemyPalette>,
  mut cards: Query<(Entity, &mut CardSlot)>,
  mut fluid: ResMut<CauldronFluid>,
  mut commands: Commands,
) {
  for CardPlayed {
    hand,
    card,
    cauldron_item,
    ..
  } in played.read()
  {
    *fluid = CauldronFluid {
      color: palette.shade(cauldron_item.color),
      icon: Some(palette.icon(cauldron_item.item_type)),
    };
    let thrown = CardSlot {
      hand: *hand,
      card: *card,
    };
    for (entity, mut slot) in &mut cards {
      if *slot == thrown {
        commands.entity(entity).despawn();
        slot.card = usize::MAX;
      // just moving cards up
      // TODO: helper to sync hand with display
      } else if player_hands.contains(*hand) && slot.hand == *hand && slot.card > *card {
        slot.card -= 1;
      }
    }
  }
}

fn update_cards(
  camera: Single<(&Projection, &Lens, Has<CameraWalk>, Has<CameraOrbit>), With<PlayerCamera>>,
  game: Single<(&Game, &Round)>,
  player_hands: Query<&PlayerCardHand>,
  time: Res<Time>,
  cards: Query<(&CardSlot, &CardPose, &mut Transform, &mut Visibility)>,
  mut huds: Query<&mut Visibility, (With<TableHud>, Without<CardSlot>)>,
) {
  let (projection, lens, walking, orbiting) = *camera;
  let (
    Game {
      player, blueprint, ..
    },
    round,
  ) = *game;
  let Projection::Perspective(perspective) = projection else {
    return;
  };
  let settled = !walking && !orbiting;
  for mut visibility in &mut huds {
    *visibility = if settled {
      Visibility::Visible
    } else {
      Visibility::Hidden
    };
  }

  let hand_size = player_hands.get(*player).map_or(0, |x| x.cards.len());
  let visible_turns = blueprint
    .levels
    .get(round.level)
    .map_or(0, |x| x.visible_turns);
  let depth = 0.6;
  let screen_height = 2.0 * depth * (perspective.fov / 2.0).tan();
  let shake = Vec3::new(-lens.shake.x, lens.shake.y, 0.0) * screen_height;
  let secs = time.elapsed_secs();
  for (slot, pose, mut transform, mut visibility) in cards {
    let (row, column, count) = if slot.hand == *player {
      (&HAND_ROW, slot.card as f32, hand_size as f32)
    } else {
      (
        &OPPONENT_ROW,
        slot.card as f32 - round.turn as f32,
        visible_turns as f32,
      )
    };
    *visibility = if settled && column >= 0.0 && column < count {
      Visibility::Visible
    } else {
      Visibility::Hidden
    };

    let from_middle = column - (count - 1.0) / 2.0;
    let bob = 0.004 * (secs * 1.1 + pose.phase).sin();
    let arc = from_middle * from_middle * row.fan * 0.3;
    let place = Vec3::new(from_middle * row.spacing, row.y - arc + bob, 0.0) * screen_height;
    transform.translation = place + shake - Vec3::Z * depth;
    let sway = Quat::from_rotation_z(0.025 * (secs * 0.8 + pose.phase * 1.7).sin());
    transform.rotation = Quat::from_rotation_z(-from_middle * row.fan)
      * Quat::from_rotation_x(-row.lean)
      * pose.tilt
      * sway;
    transform.scale = Vec3::splat(row.size * screen_height);
  }
}

fn read_keys(
  keys: Res<ButtonInput<KeyCode>>,
  games: Query<(Entity, &Round)>,
  mut turns: MessageWriter<StartPlayerTurn>,
  mut deals: MessageWriter<DealRound>,
) {
  let card_keys = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
  ];
  for (game, round) in &games {
    if round.outcome.is_some() {
      if keys.just_pressed(KeyCode::Enter) {
        deals.write(DealRound { game });
      }
      continue;
    }
    for (card, key) in card_keys.into_iter().enumerate() {
      if keys.just_pressed(key) {
        turns.write(StartPlayerTurn {
          game,
          card: Some(card),
        });
      }
    }
    if keys.just_pressed(KeyCode::Space) {
      turns.write(StartPlayerTurn { game, card: None });
    }
  }
}

fn on_card_clicked(
  click: On<PointerClick>,
  slots: Query<&CardSlot>,
  games: Query<(Entity, &Game)>,
  mut turns: MessageWriter<StartPlayerTurn>,
) {
  let Ok(slot) = slots.get(click.entity) else {
    return;
  };
  if let Some((game, _)) = games.iter().find(|(_, x)| x.player == slot.hand) {
    turns.write(StartPlayerTurn {
      game,
      card: Some(slot.card),
    });
  }
}

fn on_table_button_activated(
  activate: On<Activate>,
  actions: Query<&TableButtonAction>,
  games: Query<Entity, With<Game>>,
  mut turns: MessageWriter<StartPlayerTurn>,
  mut deals: MessageWriter<DealRound>,
  mut commands: Commands,
) {
  let Ok(action) = actions.get(activate.entity) else {
    return;
  };
  commands.trigger(GameAudioCommand::InsertOnce(
    GameAudioLibrary::ButtonEffect,
    GameAudioChannels::UI,
    Speed::Varied,
  ));
  for game in &games {
    match action {
      TableButtonAction::Pass => {
        turns.write(StartPlayerTurn { game, card: None });
      }
      TableButtonAction::Reset => {
        deals.write(DealRound { game });
      }
    }
  }
}
