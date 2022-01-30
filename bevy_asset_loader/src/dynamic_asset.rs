#[cfg(feature = "dynamic_assets")]
use bevy::asset::AssetServer;
#[cfg(feature = "dynamic_assets")]
use bevy::ecs::prelude::World;
#[cfg(feature = "dynamic_assets")]
use bevy::ecs::schedule::{State, StateData};
#[cfg(feature = "dynamic_assets")]
use bevy::utils::HashMap;

#[cfg(feature = "dynamic_assets")]
use bevy::reflect::TypeUuid;

#[cfg(feature = "dynamic_assets")]
use crate::{AssetKeys, AssetLoaderConfiguration, LoadingStatePhase};

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
#[cfg_attr(feature = "dynamic_assets", derive(serde::Deserialize))]
pub enum DynamicAsset {
    /// A dynamic asset directly loaded from a single file
    File {
        /// Asset file path
        path: String,
    },
    /// A dynamic standard material asset directly loaded from an image file
    #[cfg(feature = "render")]
    StandardMaterial {
        /// Asset file path
        path: String,
    },
    /// A dynamic texture atlas asset loaded form a sprite sheet
    #[cfg(feature = "render")]
    TextureAtlas {
        /// Asset file path
        path: String,
        /// The image width in pixels
        tile_size_x: f32,
        /// The image height in pixels
        tile_size_y: f32,
        /// Columns on the sprite sheet
        columns: usize,
        /// Rows on the sprite sheet
        rows: usize,
        /// Padding between columns in pixels
        padding_x: f32,
        /// Padding between rows in pixels
        padding_y: f32,
    },
}

impl DynamicAsset {
    /// Path to the asset file of the dynamic asset
    pub fn get_file_path(&self) -> &str {
        match self {
            DynamicAsset::File { path } => path,
            #[cfg(feature = "render")]
            DynamicAsset::StandardMaterial { path } => path,
            #[cfg(feature = "render")]
            DynamicAsset::TextureAtlas { path, .. } => path,
        }
    }
}

#[cfg(feature = "dynamic_assets")]
pub(crate) fn prepare_asset_keys<S: StateData>(world: &mut World) {
    let cell = world.cell();
    let mut asset_loader_config = cell
        .get_resource_mut::<AssetLoaderConfiguration<S>>()
        .unwrap();
    let asset_server = cell.get_resource::<AssetServer>().unwrap();
    let state = cell
        .get_resource::<State<S>>()
        .expect("Cannot get State resource");

    let files = asset_loader_config.get_asset_collection_files(state.current());
    if files.is_empty() {
        asset_loader_config
            .phase
            .insert(state.current().clone(), LoadingStatePhase::StartLoading);
        return;
    }
    for file in files {
        asset_loader_config
            .asset_collection_handles
            .push(asset_server.load(&file));
    }
    asset_loader_config.phase.insert(
        state.current().clone(),
        LoadingStatePhase::PreparingAssetKeys,
    );
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "2df82c01-9c71-4aa8-adc4-71c5824768f1"]
#[cfg(feature = "dynamic_assets")]
pub struct DynamicAssetCollection(HashMap<String, DynamicAsset>);

#[cfg(feature = "dynamic_assets")]
impl DynamicAssetCollection {
    pub fn apply(self, keys: &mut AssetKeys) {
        for (key, asset) in self.0 {
            keys.keys.insert(key, asset);
        }
    }
}
