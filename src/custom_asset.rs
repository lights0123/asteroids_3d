use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;
use bevy_rapier3d::rapier::math::Point;
use std::collections::HashMap;

pub struct CustomAssetPlugin;

impl Plugin for CustomAssetPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<CustomAsset>()
            .init_asset_loader::<CustomAssetLoader>();
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "8a1e4a4a-a8b1-40e9-8777-ca6ee96b161b"]
pub struct CustomAsset(pub(crate) Vec<(Vec<Point<f32>>, Vec<[u32; 3]>)>);

#[derive(Default)]
pub struct CustomAssetLoader;

impl AssetLoader for CustomAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let custom_asset: HashMap<String, Vec<(Vec<Point<f32>>, Vec<[u32; 3]>)>> =
                postcard::from_bytes(bytes)?;
            for (name, value) in custom_asset {
                load_context.set_labeled_asset(&name, LoadedAsset::new(CustomAsset(value)));
            }
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}
