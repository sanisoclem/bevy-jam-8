use crate::ron_asset::RonAsset;
use bevy::prelude::*;
use jam8::prelude::{
  AlchemyColor, AlchemyItemType, AlchemyLevel, AlchemyRecipe, AlchemyRuleBook, AlchemyRulesError,
  GameBlueprint, OpponentCardHand, PlayerCardHand,
};
use serde::Deserialize;

pub const BLUEPRINT_PATH: &str = "blueprints/alchemy.blueprint.ron";

#[derive(Asset, TypePath, Deserialize, Debug)]
pub struct GameBlueprintFile {
  item_types: Vec<AlchemyItemTypeFile>,
  colors: Vec<AlchemyColorFile>,
  recipes: Vec<(String, String, String)>,
  levels: Vec<AlchemyLevelFile>,
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
  turns: usize,
  visible_turns: usize,
  hand: Vec<String>,
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
  };
  let blueprint = rule_book.rules().and_then(|rules| {
    let levels = file
      .levels
      .iter()
      .map(|x| {
        Ok(AlchemyLevel {
          turns: x.turns,
          visible_turns: x.visible_turns,
          player: PlayerCardHand {
            cards: x
              .hand
              .iter()
              .map(|y| rules.item(y))
              .collect::<Result<_, _>>()?,
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
    }
    Err(error) => error!("{BLUEPRINT_PATH}: {error:?}"),
  }
}
