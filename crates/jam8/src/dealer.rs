use crate::{
  game::{OpponentCardHand, PlayerCardHand},
  rules::{AlchemyItem, AlchemyRules},
};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Dealer(pub u64);

impl Dealer {
  pub fn deal(&mut self, rules: &AlchemyRules) -> (PlayerCardHand, OpponentCardHand, AlchemyItem) {
    let items = rules.items();
    let mut player = Vec::new();
    let mut opponent = Vec::new();

    // HACK: make sure the puzzle can be solved
    // TODO: load levels via resource and should be owned by game crate
    // TODO: extract solver
    for _ in 0..1000 {
      player = (0..5).map(|_| items[self.roll(items.len())]).collect();
      opponent = (0..5)
        .map(|_| (self.roll(100) < 45).then(|| items[self.roll(items.len())]))
        .collect();
      let mut unplayed: Vec<usize> = (0..player.len()).collect();
      let mut line: Vec<AlchemyItem> = Vec::new();
      for card in &opponent {
        let pick = self.roll(unplayed.len() + 1);
        if pick < unplayed.len() {
          let item = player[unplayed.swap_remove(pick)];
          line.push(rules.combine(line.last().copied(), item));
        }
        if let Some(item) = card {
          line.push(rules.combine(line.last().copied(), *item));
        }
      }
      let played = player.len() - unplayed.len();
      let Some(&target) = line.last() else {
        continue;
      };
      let is_new = line.iter().position(|x| *x == target) == Some(line.len() - 1);
      if is_new && played >= player.len().min(2) {
        return (
          PlayerCardHand { cards: player },
          OpponentCardHand { cards: opponent },
          target,
        );
      }
    }
    let target = player[0];
    (
      PlayerCardHand { cards: player },
      OpponentCardHand { cards: opponent },
      target,
    )
  }

  fn roll(&mut self, below: usize) -> usize {
    self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mixed = (self.0 ^ (self.0 >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((mixed ^ (mixed >> 31)) % below as u64) as usize
  }
}
