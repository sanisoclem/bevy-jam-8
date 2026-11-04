use crate::rules::{AlchemyItem, AlchemyRules};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Game {
  pub player: Entity,
  pub opponent: Entity,
  pub rules: AlchemyRules,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Round {
  pub number: usize,
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
