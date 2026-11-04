use crate::{
  hand_drawn::{HandDrawn, HandDrawnSettings},
  pixel_art::{PixelArt, PixelArtSettings},
};
use bevy::prelude::*;
use serde::Deserialize;

pub fn screen_style_plugin(app: &mut App) {
  app.add_systems(Update, apply_style);
}

#[derive(Component, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[require(Camera3d)]
pub enum ScreenStyle {
  #[default]
  Off,
  PixelArt(PixelArtSettings),
  HandDrawn(HandDrawnSettings),
}

impl ScreenStyle {
  pub fn next(self) -> Self {
    match self {
      Self::Off => Self::PixelArt(default()),
      Self::PixelArt(_) => Self::HandDrawn(default()),
      Self::HandDrawn(_) => Self::Off,
    }
  }
}

fn apply_style(
  cameras: Query<(Entity, &ScreenStyle), Changed<ScreenStyle>>,
  mut commands: Commands,
) {
  for (camera, style) in cameras {
    let mut entity = commands.entity(camera);
    entity.remove::<(PixelArt, HandDrawn)>();
    match *style {
      ScreenStyle::PixelArt(settings) => {
        entity.insert(PixelArt::from(settings));
      }
      ScreenStyle::HandDrawn(settings) => {
        entity.insert(HandDrawn::from(settings));
      }
      ScreenStyle::Off => {}
    }
  }
}
