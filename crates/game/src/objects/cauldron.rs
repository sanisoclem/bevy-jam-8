use bevy::prelude::*;
use sys_asset::{AssetBarrier, IAssetBundle};

const CAULDRON_PATH: &str = "models/cauldron.glb";

pub struct CauldronAssets {
  _model: Handle<WorldAsset>,
}

impl IAssetBundle for CauldronAssets {
  fn load_all(asset_server: &AssetServer) -> (AssetBarrier, Self) {
    let (barrier, guard) = AssetBarrier::new();
    let model = asset_server
      .load_builder()
      .with_guard(guard)
      .load(GltfAssetLabel::Scene(0).from_asset(CAULDRON_PATH));
    (barrier, Self { _model: model })
  }
}

pub fn spawner(asset_server: Res<AssetServer>, mut commands: Commands) {
  commands.spawn((
    Name::new("Cauldron"),
    WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(CAULDRON_PATH))),
  ));
}
