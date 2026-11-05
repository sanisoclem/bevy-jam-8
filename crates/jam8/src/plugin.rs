use crate::{
  game::{
    CauldronCard, Game, GameBlueprint, OpponentCardHand, PlayerCardHand, Round, RoundOutcome,
  },
  rules::{AlchemyEffect, AlchemyItem},
};
use bevy::prelude::*;

pub struct Jam8Plugin;

impl Plugin for Jam8Plugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<StartGame>()
      .add_message::<StopGame>()
      .add_message::<DealRound>()
      .add_message::<StartPlayerTurn>()
      .add_message::<GameStarted>()
      .add_message::<RoundDealt>()
      .add_message::<CardPlayed>()
      .add_message::<TurnEnded>()
      .add_message::<RoundEnded>()
      .add_systems(
        Update,
        (start_games, stop_games, deal_rounds, process_player_turn)
          .chain()
          .in_set(SimSystems),
      );
  }
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimSystems;

#[derive(Message, Debug)]
pub struct StartGame {
  pub blueprint: GameBlueprint,
}

#[derive(Message, Debug)]
pub struct StopGame {
  pub game: Entity,
}

#[derive(Message, Debug)]
pub struct DealRound {
  pub game: Entity,
}

#[derive(Message, Debug)]
pub struct StartPlayerTurn {
  pub game: Entity,
  pub card: Option<usize>,
}

#[derive(Message, Debug)]
pub struct GameStarted {
  pub game: Entity,
}

#[derive(Message, Debug)]
pub struct RoundDealt {
  pub game: Entity,
}

#[derive(Message, Debug)]
pub struct CardPlayed {
  pub game: Entity,
  pub hand: Entity,
  pub card: usize, // index in hand
  pub cauldron_item: AlchemyItem,
  pub drawn: Option<AlchemyItem>,
}

#[derive(Message, Debug)]
pub struct TurnEnded {
  pub game: Entity,
}

#[derive(Message, Debug)]
pub struct RoundEnded {
  pub game: Entity,
  pub outcome: RoundOutcome,
}

fn start_games(
  mut starts: MessageReader<StartGame>,
  mut started: MessageWriter<GameStarted>,
  mut deals: MessageWriter<DealRound>,
  mut commands: Commands,
) {
  for StartGame { blueprint } in starts.read() {
    let game = commands
      .spawn((Name::new("Game"), CauldronCard::default()))
      .id();
    let player = commands
      .spawn((
        Name::new("Player"),
        PlayerCardHand::default(),
        ChildOf(game),
      ))
      .id();
    let opponent = commands
      .spawn((
        Name::new("Opponent"),
        OpponentCardHand::default(),
        ChildOf(game),
      ))
      .id();
    commands.entity(game).insert(Game {
      player,
      opponent,
      blueprint: blueprint.clone(),
    });
    started.write(GameStarted { game });
    deals.write(DealRound { game });
  }
}

fn stop_games(mut stops: MessageReader<StopGame>, mut commands: Commands) {
  for StopGame { game } in stops.read() {
    commands.entity(*game).try_despawn();
  }
}

fn deal_rounds(
  mut deals: MessageReader<DealRound>,
  games: Query<(&Game, Option<&Round>)>,
  mut dealt: MessageWriter<RoundDealt>,
  mut commands: Commands,
) {
  for DealRound { game } in deals.read() {
    let Ok((
      Game {
        player,
        opponent,
        blueprint,
      },
      round,
    )) = games.get(*game)
    else {
      continue;
    };
    let next = match round {
      None => 0,
      Some(x) if x.outcome == Some(RoundOutcome::TargetReached) => x.level + 1,
      Some(x) => x.level,
    };
    let index = if next < blueprint.levels.len() {
      next
    } else {
      0
    };
    let Some(level) = blueprint.levels.get(index) else {
      continue;
    };
    commands.entity(*game).insert((
      Round {
        level: index,
        target: level.target,
        turn: 0,
        outcome: None,
      },
      CauldronCard::default(),
    ));
    commands.entity(*player).insert(level.player.clone());
    commands.entity(*opponent).insert(level.opponent.clone());
    dealt.write(RoundDealt { game: *game });
  }
}

fn process_player_turn(
  mut turns: MessageReader<StartPlayerTurn>,
  mut games: Query<(&Game, &mut Round, &mut CauldronCard)>,
  mut player_hands: Query<&mut PlayerCardHand>,
  opponent_hands: Query<&OpponentCardHand>,
  mut thrown: MessageWriter<CardPlayed>,
  mut turns_ended: MessageWriter<TurnEnded>,
  mut ended: MessageWriter<RoundEnded>,
) {
  for StartPlayerTurn { game, card } in turns.read() {
    let Ok((
      Game {
        player,
        opponent,
        blueprint,
      },
      mut round,
      mut cauldron,
    )) = games.get_mut(*game)
    else {
      continue;
    };
    let (Ok(mut player_hand), Ok(opponent_hand)) =
      (player_hands.get_mut(*player), opponent_hands.get(*opponent))
    else {
      continue;
    };
    if round.outcome.is_some() || card.is_some_and(|x| x >= player_hand.cards.len()) {
      continue;
    }
    let rules = &blueprint.rules;
    let player_throw = card.map(|x| (x, player_hand.cards.remove(x)));
    for hand in [*player, *opponent] {
      if round.outcome.is_some() {
        break;
      }
      let throw = if hand == *player {
        player_throw
      } else if cauldron.0.and_then(|x| rules.effect(x)) == Some(AlchemyEffect::Freeze) {
        None
      } else {
        let turn = round.turn;
        round.turn += 1;
        opponent_hand
          .cards
          .get(turn)
          .copied()
          .flatten()
          .map(|x| (turn, x))
      };
      let Some((card, item)) = throw else {
        continue;
      };
      let before = cauldron.0;
      let cauldron_item = rules.combine(before, item);
      cauldron.0 = Some(cauldron_item);
      let mut drawn = None;
      if cauldron_item == round.target {
        round.outcome = Some(RoundOutcome::TargetReached);
      } else {
        match rules.effect(cauldron_item) {
          Some(AlchemyEffect::Explode) => round.outcome = Some(RoundOutcome::Exploded),
          Some(AlchemyEffect::Draw) if before != Some(cauldron_item) => {
            drawn = (!player_hand.deck.is_empty()).then(|| player_hand.deck.remove(0));
            player_hand.cards.extend(drawn);
          }
          _ => {}
        }
      }
      thrown.write(CardPlayed {
        game: *game,
        hand,
        card,
        cauldron_item,
        drawn,
      });
    }
    if round.outcome.is_none() && player_hand.cards.is_empty() {
      round.outcome = Some(RoundOutcome::OutOfCards);
    }
    turns_ended.write(TurnEnded { game: *game });
    if let Some(outcome) = round.outcome {
      ended.write(RoundEnded {
        game: *game,
        outcome,
      });
    }
  }
}
