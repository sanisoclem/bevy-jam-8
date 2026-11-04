use bevy::{
  asset::{AssetPath, io::embedded::EMBEDDED},
  prelude::*,
  shader::{ShaderRef, load_shader_library},
};
use std::path::PathBuf;

mod hand_drawn;
mod lens;
mod overlay;
mod pixel_art;
mod screen_style;
mod style_pass;

pub use hand_drawn::HandDrawnSettings;
pub use lens::Lens;
pub use overlay::{Unstyled, UnstyledLight};
pub use pixel_art::PixelArtSettings;
pub use screen_style::ScreenStyle;

pub struct SysStylePlugin;

impl Plugin for SysStylePlugin {
  fn build(&self, app: &mut App) {
    load_shader_library!(app, "noise.wesl");
    load_shader_library!(app, "screen.wesl");
    app.add_plugins((
      hand_drawn::hand_drawn_plugin,
      lens::lens_plugin,
      overlay::overlay_plugin,
      pixel_art::pixel_art_plugin,
      screen_style::screen_style_plugin,
    ));
  }
}

fn embedded_shader(path: PathBuf) -> ShaderRef {
  ShaderRef::Path(AssetPath::from_path_buf(path).with_source(EMBEDDED))
}

// use sys_style::{UnstyledLight, ScreenStyle, SysStylePlugin};
// const SCREEN_STYLE: ScreenStyle = ScreenStyle::Off;
// .add_observer(style_player_camera)
// .add_observer(light_unstyled_objects);
// fn style_player_camera(added: On<Add<PlayerCamera>>, mut commands: Commands) {
//   commands.entity(added.entity).insert(SCREEN_STYLE);
// }
//
// fn light_unstyled_objects(added: On<Add<WorldLight>>, mut commands: Commands) {
//   commands.entity(added.entity).insert(UnstyledLight);
// }
