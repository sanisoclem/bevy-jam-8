use crate::ron_asset::RonAsset;
use bevy::prelude::*;
use jam8::prelude::{AlchemyColor, AlchemyRecipe, AlchemyRuleBook};
use serde::Deserialize;

pub const RULES_PATH: &str = "rules/alchemy.rules.ron";

#[derive(Asset, TypePath, Deserialize, Debug)]
pub struct AlchemyRulesFile {
  item_types: Vec<String>,
  colors: Vec<AlchemyColorFile>,
  recipes: Vec<(String, String, String)>,
}

#[derive(Deserialize, Debug)]
struct AlchemyColorFile {
  name: String,
  light: u8,
  rgb: (f32, f32, f32),
}

impl RonAsset for AlchemyRulesFile {
  const EXTENSIONS: &'static [&'static str] = &["rules.ron"];
}

#[derive(Resource)]
pub struct AlchemyRulesHandle(pub Handle<AlchemyRulesFile>);

#[derive(Resource)]
pub struct AlchemyPalette(Vec<Color>);

impl AlchemyPalette {
  pub fn shade(&self, color: AlchemyColor) -> Color {
    self.0[color.index()]
  }
}

pub fn load_rulebook(
  mut events: MessageReader<AssetEvent<AlchemyRulesFile>>,
  handle: Res<AlchemyRulesHandle>,
  files: Res<Assets<AlchemyRulesFile>>,
  mut commands: Commands,
) {
  let loaded = events
    .read()
    .any(|x| x.is_loaded_with_dependencies(&handle.0) || x.is_modified(&handle.0));
  let Some(file) = files.get(&handle.0).filter(|_| loaded) else {
    return;
  };
  let rule_book = AlchemyRuleBook {
    item_types: file.item_types.clone(),
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
  match rule_book.rules() {
    Ok(rules) => {
      commands.insert_resource(rules);
      let palette = file
        .colors
        .iter()
        .map(|x| Color::srgb(x.rgb.0, x.rgb.1, x.rgb.2))
        .collect();
      commands.insert_resource(AlchemyPalette(palette));
    }
    Err(error) => error!("{RULES_PATH}: {error:?}"),
  }
}
