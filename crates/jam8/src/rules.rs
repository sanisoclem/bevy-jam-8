use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct AlchemyItemType(usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AlchemyColor(usize);

impl AlchemyColor {
  pub fn index(self) -> usize {
    self.0
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AlchemyItem {
  pub item_type: AlchemyItemType,
  pub color: AlchemyColor,
}

#[derive(Clone, Debug)]
pub struct AlchemyRecipe {
  pub first: String,
  pub second: String,
  pub result: String,
}

#[derive(Clone, Debug)]
pub struct AlchemyRuleBook {
  pub item_types: Vec<String>,
  pub colors: Vec<(String, u8)>,
  pub recipes: Vec<AlchemyRecipe>,
}

#[derive(Resource, Clone, Debug)]
pub struct AlchemyRules {
  item_types: Vec<String>,
  colors: Vec<(String, u8)>,
  recipes: HashMap<(AlchemyItemType, AlchemyItemType), AlchemyItemType>,
  items: Vec<AlchemyItem>,
}

#[derive(Debug)]
pub enum AlchemyRulesError {
  NoItemTypes,
  NoColors,
  InvalidColorCombination(AlchemyColor, AlchemyColor),
  UnknownItemType(String),
}

impl AlchemyRuleBook {
  pub fn rules(self) -> Result<AlchemyRules, AlchemyRulesError> {
    if self.item_types.is_empty() {
      return Err(AlchemyRulesError::NoItemTypes);
    }
    if self.colors.is_empty() {
      return Err(AlchemyRulesError::NoColors);
    }
    for first in 0..self.colors.len() {
      for second in 0..self.colors.len() {
        let light = self.colors[first].1 | self.colors[second].1;
        if !self.colors.iter().any(|(_, x)| *x == light) {
          let (first, second) = (AlchemyColor(first), AlchemyColor(second));
          return Err(AlchemyRulesError::InvalidColorCombination(first, second));
        }
      }
    }

    let item_type = |name: &str| {
      self
        .item_types
        .iter()
        .position(|x| x == name)
        .map(AlchemyItemType)
        .ok_or_else(|| AlchemyRulesError::UnknownItemType(name.into()))
    };
    let mut recipes = HashMap::default();
    for recipe in &self.recipes {
      let first = item_type(&recipe.first)?;
      let second = item_type(&recipe.second)?;
      let result = item_type(&recipe.result)?;
      recipes.insert((first, second), result);
      recipes.insert((second, first), result);
    }

    let items = (0..self.item_types.len())
      .flat_map(|x| {
        (0..self.colors.len()).map(move |y| AlchemyItem {
          item_type: AlchemyItemType(x),
          color: AlchemyColor(y),
        })
      })
      .collect();
    Ok(AlchemyRules {
      item_types: self.item_types,
      colors: self.colors,
      recipes,
      items,
    })
  }
}

impl AlchemyRules {
  pub fn item_type_name(&self, item_type: AlchemyItemType) -> &str {
    &self.item_types[item_type.0]
  }

  pub fn color_name(&self, color: AlchemyColor) -> &str {
    &self.colors[color.0].0
  }

  pub(crate) fn items(&self) -> &[AlchemyItem] {
    &self.items
  }

  pub fn combine(&self, cauldron_item: Option<AlchemyItem>, item: AlchemyItem) -> AlchemyItem {
    let Some(cauldron_item) = cauldron_item else {
      return item;
    };
    let (first, second) = (cauldron_item.item_type, item.item_type);
    let item_type = if first == second {
      first
    } else {
      self
        .recipes
        .get(&(first, second))
        .cloned()
        .unwrap_or(first.max(second))
    };
    let mixed = self.colors[cauldron_item.color.0].1 | self.colors[item.color.0].1;
    let color = self
      .colors
      .iter()
      .position(|(_, x)| *x == mixed)
      .map_or(cauldron_item.color, AlchemyColor);
    AlchemyItem { item_type, color }
  }
}
