use crate::ron_asset::ron_asset_plugin;
use bevy::{
  picking::{events::PointerClick, mesh_picking::MeshPickingPlugin},
  prelude::*,
};
use card::{CardAtlas, CardFace, CardMaker};
use jam8::prelude::*;
use rules_file::{AlchemyPalette, AlchemyRulesFile, AlchemyRulesHandle, RULES_PATH, load_rulebook};
use sys_anim::AnimateTo;
use sys_cam::PlayerCamera;

mod card;
mod rules_file;

pub use card::Card;

const PLAIN: Color = Color::srgb(0.32, 0.31, 0.33);
const HAND_ROW: CardRow = CardRow {
  height: 1.3,
  dist: 1.25,
  size: 0.22,
};
const TOP_ROW: CardRow = CardRow {
  height: 1.1,
  dist: -0.75,
  size: 0.26,
};

pub fn table_plugin(app: &mut App) {
  app
    .add_plugins((MeshPickingPlugin, ron_asset_plugin::<AlchemyRulesFile>))
    .add_systems(Startup, setup)
    .add_systems(Update, (load_rulebook, read_keys).before(SimSystems))
    .add_systems(
      Update,
      (
        display_cards.run_if(on_message::<RoundDealt>),
        update_cauldron.run_if(on_message::<CardPlayed>),
        show_status,
        snap_animations,
        face_camera,
      )
        .chain()
        .after(SimSystems),
    );
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum CardSlot {
  Hand { hand: Entity, card: usize },
  Pass { game: Entity },
  Status { game: Entity },
  Target,
  Cauldron,
}

struct CardRow {
  height: f32,
  dist: f32,
  size: f32,
}

fn setup(
  asset_server: Res<AssetServer>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands.insert_resource(AlchemyRulesHandle(asset_server.load(RULES_PATH)));
  let atlas = CardAtlas::new(&mut commands, &mut meshes, &mut images, &mut materials);
  commands.insert_resource(atlas);
}

fn item_face(
  rules: &AlchemyRules,
  palette: &AlchemyPalette,
  heading: &str,
  item: AlchemyItem,
) -> CardFace {
  let color = rules.color_name(item.color);
  let item_type = rules.item_type_name(item.item_type);
  CardFace::new(
    format!("{heading}{color}\n{item_type}"),
    palette.shade(item.color),
  )
}

fn status_text(round: &Round) -> String {
  match round.outcome {
    None => format!("Turn\n{} of {}", round.turn + 1, round.turns),
    Some(RoundOutcome::TargetReached) => "Combined".into(),
    Some(RoundOutcome::OutOfTurns) => "Out of turns".into(),
  }
}

fn display_cards(
  mut dealt: MessageReader<RoundDealt>,
  games: Query<(&Game, &Round)>,
  player_hands: Query<&PlayerCardHand>,
  opponent_hands: Query<&OpponentCardHand>,
  palette: Res<AlchemyPalette>,
  camera: Single<&GlobalTransform, With<PlayerCamera>>,
  cards: Query<(Entity, &Card)>,
  mut maker: CardMaker,
) {
  let Some(RoundDealt { game }) = dealt.read().last() else {
    return;
  };
  let Ok((
    Game {
      player,
      opponent,
      rules,
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
  for (entity, card) in &cards {
    maker.despawn(entity, card);
  }
  let dir = camera.translation().with_y(0.0).normalize_or(Vec3::Z);
  let place = |row: &CardRow, slot: usize, slots: usize| {
    let opp = (slot as f32 - (slots - 1) as f32 / 2.0) * row.size * 0.9;
    let pos = Vec3::Y.cross(dir) * opp + Vec3::Y * row.height + dir * row.dist;
    Transform::from_translation(pos).with_scale(Vec3::splat(row.size))
  };

  let slots = player_hand.cards.len() + 1;
  for (card, item) in player_hand.cards.iter().enumerate() {
    let face = item_face(rules, &palette, "", *item);
    maker
      .spawn(face, place(&HAND_ROW, card, slots))
      .insert(CardSlot::Hand {
        hand: *player,
        card,
      })
      .observe(on_card_clicked);
  }
  maker
    .spawn(
      CardFace::new("Pass", PLAIN),
      place(&HAND_ROW, slots - 1, slots),
    )
    .insert(CardSlot::Pass { game: *game })
    .observe(on_card_clicked);

  let slots = round.turns + 2;
  let status = CardFace::new(status_text(round), PLAIN);
  maker
    .spawn(status, place(&TOP_ROW, 0, slots))
    .insert(CardSlot::Status { game: *game })
    .observe(on_card_clicked);
  for (card, item) in opponent_hand.cards.iter().enumerate() {
    let heading = format!("Turn {}\n", card + 1);
    let face = match item {
      Some(item) => item_face(rules, &palette, &heading, *item),
      None => CardFace::new(format!("{heading}Nothing"), PLAIN),
    };
    maker
      .spawn(face, place(&TOP_ROW, card + 1, slots))
      .insert(CardSlot::Hand {
        hand: *opponent,
        card,
      });
  }
  let target = item_face(rules, &palette, "Target\n", round.target);
  maker
    .spawn(target, place(&TOP_ROW, slots - 1, slots))
    .insert(CardSlot::Target);
}

fn update_cauldron(
  mut thrown: MessageReader<CardPlayed>,
  player_hands: Query<(), With<PlayerCardHand>>,
  games: Query<&Game>,
  palette: Res<AlchemyPalette>,
  mut cards: Query<(Entity, &Card, &mut CardSlot)>,
  mut maker: CardMaker,
) {
  let in_cauldron = Transform::from_xyz(0.0, 1.0, 0.0).with_scale(Vec3::splat(0.34));
  let mut cauldron_card = cards
    .iter()
    .find(|(_, _, x)| **x == CardSlot::Cauldron)
    .map(|(x, _, _)| x);
  for CardPlayed {
    game,
    hand,
    card,
    cauldron_item,
  } in thrown.read()
  {
    let Ok(Game { rules, .. }) = games.get(*game) else {
      continue;
    };
    let slot = CardSlot::Hand {
      hand: *hand,
      card: *card,
    };
    let Some(entity) = cards
      .iter()
      .find(|(_, _, x)| **x == slot)
      .map(|(x, _, _)| x)
    else {
      continue;
    };
    if let Some((old, old_card, _)) = cauldron_card.and_then(|x| cards.get(x).ok()) {
      maker.despawn(old, old_card);
    }
    let Ok((_, thrown, mut slot)) = cards.get_mut(entity) else {
      continue;
    };
    maker.relabel(thrown, item_face(rules, &palette, "", *cauldron_item));
    *slot = CardSlot::Cauldron;
    maker.commands.entity(entity).insert(AnimateTo {
      to: in_cauldron,
      secs: 0.4,
    });
    cauldron_card = Some(entity);
    // just moving cards up
    // TODO: helper to sync hand with display
    if player_hands.contains(*hand) {
      for (_, _, mut slot) in &mut cards {
        if let CardSlot::Hand { hand: x, card: y } = &mut *slot
          && *x == *hand
          && *y > *card
        {
          *y -= 1;
        }
      }
    }
  }
}

fn show_status(
  rounds: Query<(Entity, &Round), Changed<Round>>,
  cards: Query<(&Card, &CardSlot)>,
  mut maker: CardMaker,
) {
  for (game, round) in &rounds {
    for (card, slot) in &cards {
      if *slot == (CardSlot::Status { game }) {
        maker.relabel(card, CardFace::new(status_text(round), PLAIN));
      }
    }
  }
}

fn snap_animations(
  animations: Query<(Entity, &AnimateTo, &mut Transform)>,
  mut commands: Commands,
) {
  for (entity, animate_to, mut transform) in animations {
    *transform = animate_to.to;
    // TODO:
    commands.entity(entity).remove::<AnimateTo>();
  }
}

fn face_camera(
  camera: Single<&GlobalTransform, With<PlayerCamera>>,
  cards: Query<&mut Transform, With<Card>>,
) {
  for mut card in cards {
    card.rotation = camera.rotation();
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
  games: Query<(Entity, &Game, &Round)>,
  mut turns: MessageWriter<StartPlayerTurn>,
  mut deals: MessageWriter<DealRound>,
) {
  match slots.get(click.entity) {
    Ok(CardSlot::Hand { hand, card }) => {
      if let Some((game, _, _)) = games.iter().find(|(_, x, _)| x.player == *hand) {
        turns.write(StartPlayerTurn {
          game,
          card: Some(*card),
        });
      }
    }
    Ok(CardSlot::Pass { game }) => {
      turns.write(StartPlayerTurn {
        game: *game,
        card: None,
      });
    }
    Ok(CardSlot::Status { game })
      if games.get(*game).is_ok_and(|(_, _, x)| x.outcome.is_some()) =>
    {
      deals.write(DealRound { game: *game });
    }
    _ => {}
  }
}
