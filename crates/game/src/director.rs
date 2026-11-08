use crate::{
  caption::Dialogue,
  objects::CauldronFluid,
  table::{
    AlchemyPalette, CARD_ASPECT, CardMaker, CardPlace, CardSlot, HUD_MARGIN_VH, Overlay,
    RecipeNote, TARGET_HUD_VH, TableHud, TableScript, TableView, TargetHud,
  },
};
use bevy::{
  picking::{events::PointerClick, hover::Hovered},
  prelude::*,
  ui_widgets::{Activate, Button},
};
use jam8::prelude::*;
use std::collections::VecDeque;
use sys_audio::{GameAudioChannels, GameAudioCommand, GameAudioLibrary, Speed};
use sys_cam::{CameraOrbit, CameraWalk, PlayerCamera};

// all coordination work (non music) goes here
pub fn director_plugin(app: &mut App) {
  app
    .add_systems(Update, (read_keys, skip).before(SimSystems))
    .add_systems(
      Update,
      (react, direct)
        .chain()
        .distributive_run_if(any_with_component::<Director>)
        .after(SimSystems),
    )
    .add_observer(on_director_added)
    .add_observer(on_table_btn);
}

#[derive(Component, Debug, Default)]
pub struct Director {
  game: Option<Entity>,
  steps: VecDeque<DirectorStep>,
  elapsed: f32,
  idle: bool,
  frozen: bool,
}

#[derive(Debug)]
enum DirectorStep {
  Settle, // waiting for cam to settle
  Say(String),
  Begin,
  Wait(f32),
  Fade(f32), // fade overlay
  Showcase,
  Stash,
  SetTable,
  Deal,
  Throw(CardSlot),
  Sink(AlchemyItem),
  Advance { turn: usize, frozen: bool },
  Clear,
  Redeal,
  Finish(String),
}

#[derive(Component, Debug)]
enum TableButtonAction {
  Pass,
  Reset,
}

fn on_director_added(
  added: On<Add<Director>>,
  camera: Single<Entity, With<PlayerCamera>>,
  script: Res<TableScript>,
  mut directors: Query<&mut Director>,
  mut commands: Commands,
) {
  let Ok(mut director) = directors.get_mut(added.entity) else {
    return;
  };
  director.steps.push_back(DirectorStep::Settle);
  director
    .steps
    .extend(script.intro.iter().cloned().map(DirectorStep::Say));
  director.steps.push_back(DirectorStep::Begin);

  let view = commands
    .spawn((
      TableView::default(),
      Transform::default(),
      Visibility::default(),
      ChildOf(*camera),
    ))
    .id();
  commands.spawn((RecipeNote::default(), ChildOf(view)));

  // used to hide the scene
  commands.spawn((
    Overlay::default(),
    Node {
      width: percent(100),
      height: percent(100),
      ..default()
    },
    BackgroundColor(Color::NONE),
    GlobalZIndex(1),
    Pickable::IGNORE,
  ));

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

  commands.spawn((
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
        TargetHud::default(),
        Hovered::default(),
        Node {
          position_type: PositionType::Absolute,
          top: Val::Vh(HUD_MARGIN_VH),
          right: Val::Vh(HUD_MARGIN_VH),
          width: Val::Vh(TARGET_HUD_VH * CARD_ASPECT),
          height: Val::Vh(TARGET_HUD_VH),
          ..default()
        },
        UiTransform::from_scale(Vec2::ZERO),
        children![(
          Node {
            position_type: PositionType::Absolute,
            top: percent(100),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
          },
          children![label("Target")],
        )],
      ),
      (
        Node {
          position_type: PositionType::Absolute,
          right: Val::Vh(HUD_MARGIN_VH),
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

fn react(
  mut started: MessageReader<GameStarted>,
  mut dealt: MessageReader<RoundDealt>,
  mut played: MessageReader<CardPlayed>,
  mut turns: MessageReader<TurnEnded>,
  mut ended: MessageReader<RoundEnded>,
  games: Query<(&Game, &Round, &CauldronCard)>,
  opponent_hands: Query<&OpponentCardHand>,
  script: Res<TableScript>,
  director: Single<&mut Director>,
) {
  use DirectorStep::*;
  let mut director = director.into_inner();

  for GameStarted { game } in started.read() {
    director.game = Some(*game);
  }

  for RoundDealt { game } in dealt.read() {
    let Ok((_, round, _)) = games.get(*game) else {
      continue;
    };
    director.frozen = false;
    director.steps.clear();
    director.steps.extend([SetTable, Fade(0.0), Wait(0.4)]);
    if let Some(level) = script.levels.get(round.level) {
      director.steps.extend(level.lines.iter().cloned().map(Say));
      director.steps.extend([
        Showcase,
        Wait(0.6),
        Say(level.goal.clone()),
        // Wait(0.6),
        Stash,
        Wait(0.6),
      ]);
    }
    director.steps.push_back(Deal);
  }

  for CardPlayed {
    hand,
    card,
    cauldron_item,
    drawn,
    ..
  } in played.read()
  {
    director.steps.extend([
      Throw(CardSlot {
        hand: *hand,
        card: *card,
      }),
      Wait(0.4),
      Sink(*cauldron_item),
      Wait(1.1),
    ]);
    if drawn.is_some() {
      director
        .steps
        .extend([Deal, Wait(0.4), Say(script.card_drawn.clone())]);
    }
  }

  for TurnEnded { game } in turns.read() {
    let Ok((
      Game {
        opponent,
        blueprint,
        ..
      },
      round,
      cauldron,
    )) = games.get(*game)
    else {
      continue;
    };
    let frozen = cauldron.0.and_then(|x| blueprint.rules.effect(x)) == Some(AlchemyEffect::Freeze);
    director.steps.push_back(Advance {
      turn: round.turn,
      frozen,
    });
    let waiting = opponent_hands
      .get(*opponent)
      .is_ok_and(|x| x.cards.iter().skip(round.turn).any(Option::is_some));
    if frozen && !director.frozen && waiting && round.outcome.is_none() {
      director.steps.push_back(Say(script.frozen.clone()));
    }
    director.frozen = frozen;
  }

  for RoundEnded { game, outcome } in ended.read() {
    let Ok((Game { blueprint, .. }, round, _)) = games.get(*game) else {
      continue;
    };
    if *outcome == RoundOutcome::TargetReached {
      director.steps.extend([
        Wait(0.3),
        Clear,
        // Wait(10.6),
        Showcase,
        Wait(0.6),
        Say(script.won.clone()),
        Fade(1.0),
        Wait(1.2),
      ]);
      if round.level + 1 == blueprint.levels.len() {
        director.steps.push_back(Finish(script.finished.clone()));
      } else {
        director.steps.push_back(Redeal);
      }
    } else {
      let line = if *outcome == RoundOutcome::Exploded {
        &script.exploded
      } else {
        &script.out_of_cards
      };
      director.steps.extend([
        Wait(0.3),
        Fade(0.6),
        Wait(0.4),
        Say(line.clone()),
        Clear,
        Wait(0.5),
        Redeal,
      ]);
    }
  }
}

fn direct(
  director: Single<&mut Director>,
  time: Res<Time>,
  (camera, texts): (
    Single<(Has<CameraWalk>, Has<CameraOrbit>), With<PlayerCamera>>,
    Query<(), With<Dialogue>>,
  ),
  (games, player_hands, opponent_hands): (
    Query<(&Game, &Round)>,
    Query<&PlayerCardHand>,
    Query<&OpponentCardHand>,
  ),
  mut cards: Query<(Entity, Option<&mut CardSlot>, &mut CardPlace, &mut Pickable)>,
  (view, mut hud, target_hud, mut veil): (
    Single<(Entity, &mut TableView)>,
    Single<&mut Visibility, With<TableHud>>,
    Single<(Entity, &mut TargetHud)>,
    Single<&mut Overlay>,
  ),
  (palette, blueprint): (Res<AlchemyPalette>, Res<GameBlueprint>),
  mut fluid: ResMut<CauldronFluid>,
  (mut starts, mut deals): (MessageWriter<StartGame>, MessageWriter<DealRound>),
  mut maker: CardMaker,
) {
  let mut director = director.into_inner();
  let (view_entity, mut view) = view.into_inner();
  let (target_hud_entity, mut target_hud) = target_hud.into_inner();
  let (walking, orbiting) = *camera;
  let round = director.game.and_then(|x| games.get(x).ok());
  director.elapsed += time.delta_secs();

  // only progress if no text is displayed
  while texts.is_empty()
    && let Some(step) = director.steps.pop_front()
  {
    let elapsed = director.elapsed;
    let said = matches!(step, DirectorStep::Say(_));
    match step {
      DirectorStep::Settle if walking || orbiting => {
        director.steps.push_front(step);
        break;
      }
      DirectorStep::Wait(secs) if elapsed < secs => {
        director.steps.push_front(step);
        break;
      }
      DirectorStep::Settle | DirectorStep::Wait(_) => {}
      DirectorStep::Say(text) => {
        maker.commands.spawn(Dialogue::new(text));
      }
      DirectorStep::Begin => {
        starts.write(StartGame {
          blueprint: blueprint.clone(),
        });
      }
      DirectorStep::Fade(alpha) => veil.alpha = alpha,
      DirectorStep::Showcase => {
        for (.., mut place, _) in &mut cards {
          if *place == CardPlace::Corner {
            *place = CardPlace::Showcase;
          }
        }
        target_hud.shown = false;
      }
      DirectorStep::Stash => {
        for (.., mut place, _) in &mut cards {
          if *place == CardPlace::Showcase {
            *place = CardPlace::Corner;
          }
        }
        target_hud.shown = true;
        **hud = Visibility::Inherited;
      }
      DirectorStep::SetTable => {
        let Some((Game { blueprint, .. }, round)) = round else {
          continue;
        };
        for (entity, ..) in &cards {
          maker.commands.entity(entity).despawn();
        }
        fluid.splash(
          CauldronFluid::default().color,
          None,
          time.elapsed_secs_wrapped(),
        );
        *view = TableView {
          turn: 0,
          visible_turns: blueprint
            .levels
            .get(round.level)
            .map_or(0, |x| x.visible_turns),
          frozen: false,
          settled: false,
        };
        let face = palette.face(round.target);
        let target = maker.hud(face);
        maker.commands.entity(target_hud_entity).insert(target);
        maker
          .spawn(face)
          .insert((CardPlace::Corner, Pickable::IGNORE, ChildOf(view_entity)));
      }
      DirectorStep::Deal => {
        let Some((
          Game {
            player, opponent, ..
          },
          round,
        )) = round
        else {
          continue;
        };
        let (Ok(player_hand), Ok(opponent_hand)) =
          (player_hands.get(*player), opponent_hands.get(*opponent))
        else {
          continue;
        };
        let shown = cards
          .iter()
          .filter(|(_, _, place, _)| **place != CardPlace::Pot)
          .filter_map(|(_, slot, ..)| slot.copied())
          .collect::<Vec<_>>();
        let players = player_hand.cards.iter().enumerate().map(|(x, item)| {
          let slot = CardSlot {
            hand: *player,
            card: x,
          };
          (slot, *item, x as f32 * 0.08)
        });
        let opponents = opponent_hand
          .cards
          .iter()
          .enumerate()
          .skip(round.turn)
          .filter_map(|(x, item)| {
            let slot = CardSlot {
              hand: *opponent,
              card: x,
            };
            item.map(|y| (slot, y, 0.25 + (x - round.turn) as f32 * 0.08))
          });
        let missing = players
          .chain(opponents)
          .filter(|(slot, ..)| !shown.contains(slot))
          .collect::<Vec<_>>();
        if missing.is_empty() {
          continue;
        }
        for (slot, item, _) in missing.into_iter().filter(|(.., x)| *x <= elapsed) {
          let mut card = maker.spawn(palette.face(item));
          card.insert((slot, CardPlace::Row, ChildOf(view_entity)));
          if slot.hand == *player {
            card
              .insert((Pickable::default(), Hovered::default()))
              .observe(on_card_clicked);
          } else {
            card.insert((Pickable::IGNORE, Hovered::default()));
          }
        }
        director.steps.push_front(step);
        break;
      }
      DirectorStep::Throw(thrown) => {
        let mine = round.is_some_and(|(x, _)| x.player == thrown.hand);
        for (_, slot, mut place, _) in &mut cards {
          let Some(mut slot) = slot.filter(|x| x.hand == thrown.hand) else {
            continue;
          };
          if *place != CardPlace::Row {
            continue;
          }
          if *slot == thrown {
            *place = CardPlace::Pot;
          } else if mine && slot.card > thrown.card {
            slot.card -= 1;
          }
        }
      }
      DirectorStep::Sink(item) => {
        for (entity, _, place, _) in &cards {
          if *place == CardPlace::Pot {
            maker.commands.entity(entity).despawn();
          }
        }
        fluid.splash(
          palette.shade(item.color),
          Some(palette.icon(item.item_type)),
          time.elapsed_secs_wrapped(),
        );
      }
      DirectorStep::Advance { turn, frozen } => {
        view.turn = turn;
        view.frozen = frozen;
      }
      DirectorStep::Clear => {
        for (_, slot, mut place, _) in &mut cards {
          if slot.is_some() {
            *place = CardPlace::Away;
          }
        }
      }
      DirectorStep::Redeal => {
        if let Some(game) = director.game {
          deals.write(DealRound { game });
        }
      }
      DirectorStep::Finish(text) => {
        maker.commands.spawn((
          Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
          },
          BackgroundColor(Color::BLACK),
          GlobalZIndex(2),
          children![(
            Text::new(text),
            TextFont {
              font_size: FontSize::Vh(6.0),
              ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
          )],
        ));
      }
    }
    director.elapsed = 0.0;
    if said {
      break;
    }
  }

  director.idle = director.steps.is_empty()
    && texts.is_empty()
    && round.is_some_and(|(_, x)| x.outcome.is_none());
  view.settled = director.idle;
  for (_, slot, place, mut pickable) in &mut cards {
    let hoverable = director.idle && *place == CardPlace::Row && slot.is_some();
    if pickable.is_hoverable != hoverable {
      pickable.is_hoverable = hoverable;
    }
  }
}

fn read_keys(
  keys: Res<ButtonInput<KeyCode>>,
  director: Single<&mut Director>,
  mut turns: MessageWriter<StartPlayerTurn>,
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
  let mut director = director.into_inner();
  let Some(game) = director.game.filter(|_| director.idle) else {
    return;
  };
  let card = card_keys.into_iter().position(|x| keys.just_pressed(x));
  if card.is_some() || keys.just_pressed(KeyCode::Space) {
    turns.write(StartPlayerTurn { game, card });
    director.idle = false;
  }
}

fn skip(mouse: Res<ButtonInput<MouseButton>>, director: Single<&mut Director>) {
  if !mouse.just_pressed(MouseButton::Left) {
    return;
  }
  let mut director = director.into_inner();
  let end = director
    .steps
    .iter()
    .position(|x| {
      !matches!(
        x,
        DirectorStep::Wait(_)
          | DirectorStep::Sink(_)
          | DirectorStep::Deal
          | DirectorStep::Advance { .. }
      )
    })
    .unwrap_or(director.steps.len());
  let rest = director.steps.split_off(end);
  director
    .steps
    .retain(|x| !matches!(x, DirectorStep::Wait(_)));
  director.steps.extend(rest);
}

fn on_card_clicked(
  click: On<PointerClick>,
  slots: Query<&CardSlot>,
  director: Single<&mut Director>,
  mut turns: MessageWriter<StartPlayerTurn>,
) {
  let mut director = director.into_inner();
  let (Ok(slot), Some(game)) = (slots.get(click.entity), director.game) else {
    return;
  };
  if director.idle {
    turns.write(StartPlayerTurn {
      game,
      card: Some(slot.card),
    });
    director.idle = false;
  }
}

fn on_table_btn(
  activate: On<Activate>,
  actions: Query<&TableButtonAction>,
  director: Single<&mut Director>,
  mut turns: MessageWriter<StartPlayerTurn>,
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
  let mut director = director.into_inner();
  let Some(game) = director.game.filter(|_| director.idle) else {
    return;
  };
  match action {
    TableButtonAction::Pass => {
      turns.write(StartPlayerTurn { game, card: None });
    }
    TableButtonAction::Reset => {
      director.steps.extend([
        DirectorStep::Clear,
        DirectorStep::Wait(0.4),
        DirectorStep::Redeal,
      ]);
    }
  }
  director.idle = false;
}
