use crate::{
  dealer::Dealer,
  game::{CauldronCard, Game, OpponentCardHand, PlayerCardHand, Round, RoundOutcome},
  rules::{AlchemyItem, AlchemyRules},
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
  pub seed: u64,
  pub rules: AlchemyRules,
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
  for StartGame { seed, rules } in starts.read() {
    let game = commands
      .spawn((Name::new("Game"), Dealer(*seed), CauldronCard::default()))
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
      rules: rules.clone(),
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
  mut games: Query<(&Game, &mut Dealer, Option<&Round>)>,
  mut dealt: MessageWriter<RoundDealt>,
  mut commands: Commands,
) {
  for DealRound { game } in deals.read() {
    let Ok((
      Game {
        player,
        opponent,
        rules,
      },
      mut dealer,
      round,
    )) = games.get_mut(*game)
    else {
      continue;
    };
    let (player_hand, opponent_hand, target) = dealer.deal(rules);
    commands.entity(*game).insert((
      Round {
        number: round.map_or(1, |x| x.number + 1),
        target,
        turn: 0,
        turns: opponent_hand.cards.len(),
        outcome: None,
      },
      CauldronCard::default(),
    ));
    commands.entity(*player).insert(player_hand);
    commands.entity(*opponent).insert(opponent_hand);
    dealt.write(RoundDealt { game: *game });
  }
}

fn process_player_turn(
  mut turns: MessageReader<StartPlayerTurn>,
  mut games: Query<(&Game, &mut Round, &mut CauldronCard)>,
  mut player_hands: Query<&mut PlayerCardHand>,
  opponent_hands: Query<&OpponentCardHand>,
  mut thrown: MessageWriter<CardPlayed>,
  mut ended: MessageWriter<RoundEnded>,
) {
  for StartPlayerTurn { game, card } in turns.read() {
    let Ok((
      Game {
        player,
        opponent,
        rules,
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
    let player_throw = card.map(|x| (*player, x, player_hand.cards.remove(x)));
    let opponent_throw = opponent_hand
      .cards
      .get(round.turn)
      .copied()
      .flatten()
      .map(|x| (*opponent, round.turn, x));
    for (hand, card, item) in player_throw.into_iter().chain(opponent_throw) {
      if round.outcome.is_some() {
        break;
      }
      let cauldron_item = rules.combine(cauldron.0, item);
      cauldron.0 = Some(cauldron_item);
      if cauldron_item == round.target {
        round.outcome = Some(RoundOutcome::TargetReached);
      }
      thrown.write(CardPlayed {
        game: *game,
        hand,
        card,
        cauldron_item,
      });
    }
    if round.outcome.is_none() {
      round.turn += 1;
      if round.turn == round.turns {
        round.outcome = Some(RoundOutcome::OutOfTurns);
      }
    }
    if let Some(outcome) = round.outcome {
      ended.write(RoundEnded {
        game: *game,
        outcome,
      });
    }
  }
}
