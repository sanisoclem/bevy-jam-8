use bevy::prelude::*;
use game::{GamePlugin, gyms::Gym};

#[cfg(feature = "dev")]
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_wasm() {
  main();
}

fn main() {
  let gym = match std::env::args().nth(1).as_deref() {
    Some("look_around") => Gym::LookAround,
    _ => Gym::Main,
  };

  let mut app = App::new();

  app.add_plugins(GamePlugin { gym });

  #[cfg(feature = "dev")]
  app
    .add_plugins(EguiPlugin::default())
    .init_schedule(EguiPrimaryContextPass);

  app.run();
}
