mod game;
pub mod plugin;
mod rules;

pub mod prelude {
  pub use crate::{game::*, plugin::*, rules::*};
}
