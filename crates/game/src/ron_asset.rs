use bevy::{
  asset::{AssetLoader, LoadContext, io::Reader},
  prelude::*,
};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

pub trait RonAsset: Asset + DeserializeOwned {
  const EXTENSIONS: &'static [&'static str];
}

pub fn ron_asset_plugin<A: RonAsset>(app: &mut App) {
  app.init_asset::<A>().init_asset_loader::<RonLoader<A>>();
}

#[derive(TypePath)]
struct RonLoader<A>(PhantomData<fn() -> A>);

impl<A> Default for RonLoader<A> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<A: RonAsset> AssetLoader for RonLoader<A> {
  type Asset = A;
  type Settings = ();
  type Error = BevyError;

  async fn load(
    &self,
    reader: &mut dyn Reader,
    _settings: &(),
    _context: &mut LoadContext<'_>,
  ) -> Result<A, BevyError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    Ok(ron::de::from_bytes(&bytes)?)
  }

  fn extensions(&self) -> &[&str] {
    A::EXTENSIONS
  }
}
