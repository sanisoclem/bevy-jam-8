use crate::{
  game::{CauldronCard, OpponentCardHand, PlayerCardHand, Round, RoundOutcome},
  rules::{AlchemyEffect, AlchemyItem, AlchemyItemType, AlchemyRules},
};
use bevy::platform::collections::HashMap;
use std::ops::ControlFlow::{self, Break, Continue};

// relevant recipes for making the target card
pub fn get_relevant_recipes(
  rules: &AlchemyRules,
  round: &Round,
  cauldron: &CauldronCard,
  player: &PlayerCardHand,
  opponent: &OpponentCardHand,
) -> Vec<[AlchemyItemType; 3]> {
  let mut search = TargetSearch {
    rules,
    target: round.target,
    cards: [player.cards.as_slice(), &player.deck].concat(),
    first_draw: player.cards.len(),
    opponent: opponent.cards.get(round.turn..).unwrap_or_default(),
    found: HashMap::default(),
  };
  let table = Table {
    cauldron_card: cauldron.0,
    hand: (1 << player.cards.len()) - 1,
    drawn: 0,
    turn: 0,
  };
  search.recipes(table).unwrap_or_default()
}

struct TargetSearch<'a> {
  rules: &'a AlchemyRules,
  target: AlchemyItem,
  cards: Vec<AlchemyItem>,
  first_draw: usize,
  opponent: &'a [Option<AlchemyItem>],
  found: HashMap<Table, Option<Vec<[AlchemyItemType; 3]>>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Table {
  cauldron_card: Option<AlchemyItem>,
  hand: u32,
  drawn: usize,
  turn: usize,
}

impl TargetSearch<'_> {
  fn recipes(&mut self, table: Table) -> Option<Vec<[AlchemyItemType; 3]>> {
    if let Some(x) = self.found.get(&table) {
      return x.clone();
    }
    self.found.insert(table, None);
    let mut found: Option<Vec<_>> = None;
    let cards = (0..self.cards.len()).filter(|x| table.hand & 1 << x != 0);
    for card in cards.map(Some).chain([None]) {
      let mut used = Vec::new();
      let rest = match self.turn(table, card, &mut used) {
        Break(RoundOutcome::TargetReached) => Some(Vec::new()),
        Break(_) => None,
        Continue(next) => self.recipes(next),
      };
      if let Some(rest) = rest {
        found
          .get_or_insert_default()
          .extend(used.into_iter().chain(rest));
      }
    }
    if let Some(x) = &mut found {
      x.sort();
      x.dedup();
    }
    self.found.insert(table, found.clone());
    found
  }

  fn turn(
    &self,
    table: Table,
    card: Option<usize>,
    used: &mut Vec<[AlchemyItemType; 3]>,
  ) -> ControlFlow<RoundOutcome, Table> {
    let mut table = table;
    if let Some(x) = card {
      table.hand &= !(1 << x);
      self.throw(&mut table, self.cards[x], used)?;
    }
    let frozen =
      table.cauldron_card.and_then(|x| self.rules.effect(x)) == Some(AlchemyEffect::Freeze);
    if !frozen {
      if let Some(item) = self.opponent.get(table.turn).copied().flatten() {
        self.throw(&mut table, item, used)?;
      }
      table.turn = (table.turn + 1).min(self.opponent.len());
    }
    if table.hand == 0 {
      Break(RoundOutcome::OutOfCards)
    } else {
      Continue(table)
    }
  }

  fn throw(
    &self,
    table: &mut Table,
    item: AlchemyItem,
    used: &mut Vec<[AlchemyItemType; 3]>,
  ) -> ControlFlow<RoundOutcome> {
    if let Some(x) = table.cauldron_card
      && x.item_type != item.item_type
    {
      let (a, b) = (x.item_type, item.item_type);
      let (first, second) = (a.min(b), a.max(b));
      used.push([first, second, self.rules.combine_types(first, second)]);
    }
    let landed = self.rules.combine(table.cauldron_card, item);
    let before = table.cauldron_card.replace(landed);
    if landed == self.target {
      return Break(RoundOutcome::TargetReached);
    }
    match self.rules.effect(landed) {
      Some(AlchemyEffect::Explode) => Break(RoundOutcome::Exploded),
      Some(AlchemyEffect::Draw)
        if before != Some(landed) && self.first_draw + table.drawn < self.cards.len() =>
      {
        table.hand |= 1 << (self.first_draw + table.drawn);
        table.drawn += 1;
        Continue(())
      }
      _ => Continue(()),
    }
  }
}
