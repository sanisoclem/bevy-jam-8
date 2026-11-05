use crate::rules::{AlchemyItem, AlchemyRules};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Game {
  pub player: Entity,
  pub opponent: Entity,
  pub blueprint: GameBlueprint,
}

#[derive(Resource, Clone, Debug)]
pub struct GameBlueprint {
  pub rules: AlchemyRules,
  pub levels: Vec<AlchemyLevel>,
}

#[derive(Clone, Debug)]
pub struct AlchemyLevel {
  pub turns: usize,
  pub visible_turns: usize,
  pub player: PlayerCardHand,
  pub opponent: OpponentCardHand,
  pub target: AlchemyItem,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Round {
  pub level: usize,
  pub target: AlchemyItem,
  pub turn: usize,
  pub turns: usize,
  pub outcome: Option<RoundOutcome>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoundOutcome {
  TargetReached,
  OutOfTurns,
}

#[derive(Component, Clone, Debug, Default)]
pub struct PlayerCardHand {
  pub cards: Vec<AlchemyItem>,
}

#[derive(Component, Clone, Debug, Default)]
pub struct OpponentCardHand {
  pub cards: Vec<Option<AlchemyItem>>,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CauldronCard(pub Option<AlchemyItem>);
