use super::CardFace;
use crate::ron_asset::RonAsset;
use bevy::prelude::*;
use jam8::prelude::{
  AlchemyColor, AlchemyEffect, AlchemyItem, AlchemyItemType, AlchemyLevel, AlchemyRecipe,
  AlchemyRuleBook, AlchemyRulesError, GameBlueprint, OpponentCardHand, PlayerCardHand,
};
use serde::Deserialize;

pub const BLUEPRINT_PATH: &str = "blueprints/alchemy.blueprint.ron";

#[derive(Asset, TypePath, Deserialize, Debug)]
pub struct GameBlueprintFile {
  item_types: Vec<AlchemyItemTypeFile>,
  colors: Vec<AlchemyColorFile>,
  recipes: Vec<(String, String, String)>,
  effects: AlchemyEffectsFile,
  script: TableScript,
  levels: Vec<AlchemyLevelFile>,
}

#[derive(Deserialize, Debug)]
struct AlchemyEffectsFile {
  freeze: String,
  explode: String,
  draw: String,
}

#[derive(Resource, Deserialize, Clone, Debug)]
pub struct TableScript {
  pub intro: Vec<String>,
  pub won: String,
  pub finished: String,
  pub out_of_cards: String,
  pub exploded: String,
  #[serde(skip)]
  pub levels: Vec<LevelScript>,
}

#[derive(Clone, Debug)]
pub struct LevelScript {
  pub lines: Vec<String>,
  pub goal: String,
}

#[derive(Deserialize, Debug)]
struct AlchemyItemTypeFile {
  name: String,
  icon: u32,
}

#[derive(Deserialize, Debug)]
struct AlchemyColorFile {
  name: String,
  light: u8,
  rgb: (f32, f32, f32),
}

#[derive(Deserialize, Debug)]
struct AlchemyLevelFile {
  #[serde(default)]
  lines: Vec<String>,
  goal: String,
  #[serde(default)]
  visible_turns: usize,
  hand: Vec<String>,
  #[serde(default)]
  deck: Vec<String>,
  #[serde(default)]
  opponent: Vec<Option<String>>,
  target: String,
}

impl RonAsset for GameBlueprintFile {
  const EXTENSIONS: &'static [&'static str] = &["blueprint.ron"];
}

#[derive(Resource)]
pub struct GameBlueprintHandle(pub Handle<GameBlueprintFile>);

#[derive(Resource)]
pub struct AlchemyPalette {
  shades: Vec<Color>,
  icons: Vec<u32>,
}

impl AlchemyPalette {
  pub fn shade(&self, color: AlchemyColor) -> Color {
    self.shades[color.index()]
  }

  pub fn icon(&self, item_type: AlchemyItemType) -> u32 {
    self.icons[item_type.index()]
  }

  pub fn face(&self, item: AlchemyItem) -> CardFace {
    CardFace {
      shade: self.shade(item.color),
      icon: self.icon(item.item_type),
    }
  }
}

pub fn load_blueprint(
  mut events: MessageReader<AssetEvent<GameBlueprintFile>>,
  handle: Res<GameBlueprintHandle>,
  files: Res<Assets<GameBlueprintFile>>,
  mut commands: Commands,
) {
  let loaded = events
    .read()
    .any(|x| x.is_loaded_with_dependencies(&handle.0) || x.is_modified(&handle.0));
  let Some(file) = files.get(&handle.0).filter(|_| loaded) else {
    return;
  };
  let rule_book = AlchemyRuleBook {
    item_types: file.item_types.iter().map(|x| x.name.clone()).collect(),
    colors: file
      .colors
      .iter()
      .map(|x| (x.name.clone(), x.light))
      .collect(),
    recipes: file
      .recipes
      .iter()
      .map(|(first, second, result)| AlchemyRecipe {
        first: first.clone(),
        second: second.clone(),
        result: result.clone(),
      })
      .collect(),
    effects: vec![
      (file.effects.freeze.clone(), AlchemyEffect::Freeze),
      (file.effects.explode.clone(), AlchemyEffect::Explode),
      (file.effects.draw.clone(), AlchemyEffect::Draw),
    ],
  };
  let blueprint = rule_book.rules().and_then(|rules| {
    let items = |names: &[String]| {
      names
        .iter()
        .map(|x| rules.item(x))
        .collect::<Result<Vec<_>, _>>()
    };
    let levels = file
      .levels
      .iter()
      .map(|x| {
        Ok(AlchemyLevel {
          visible_turns: x.visible_turns,
          player: PlayerCardHand {
            cards: items(&x.hand)?,
            deck: items(&x.deck)?,
          },
          opponent: OpponentCardHand {
            cards: x
              .opponent
              .iter()
              .map(|y| y.as_deref().map(|z| rules.item(z)).transpose())
              .collect::<Result<_, _>>()?,
          },
          target: rules.item(&x.target)?,
        })
      })
      .collect::<Result<Vec<_>, AlchemyRulesError>>()?;
    Ok(GameBlueprint { rules, levels })
  });
  match blueprint {
    Ok(blueprint) => {
      commands.insert_resource(blueprint);
      commands.insert_resource(AlchemyPalette {
        shades: file
          .colors
          .iter()
          .map(|x| Color::srgb(x.rgb.0, x.rgb.1, x.rgb.2))
          .collect(),
        icons: file.item_types.iter().map(|x| x.icon).collect(),
      });
      commands.insert_resource(TableScript {
        levels: file
          .levels
          .iter()
          .map(|x| LevelScript {
            lines: x.lines.clone(),
            goal: x.goal.clone(),
          })
          .collect(),
        ..file.script.clone()
      });
    }
    Err(error) => error!("{BLUEPRINT_PATH}: {error:?}"),
  }
}
