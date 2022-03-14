#[cfg(feature = "dynamic_assets")]
use bevy::asset::AssetServer;
use bevy::asset::{Assets, LoadState};
#[cfg(feature = "dynamic_assets")]
use bevy::ecs::prelude::World;
#[cfg(feature = "dynamic_assets")]
use bevy::ecs::schedule::{State, StateData};
#[cfg(feature = "dynamic_assets")]
use bevy::utils::HashMap;

#[cfg(feature = "dynamic_assets")]
use bevy::reflect::TypeUuid;

#[cfg(feature = "dynamic_assets")]
use crate::{AssetLoaderConfiguration, DynamicAssets, LoadingState};

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
#[cfg_attr(feature = "dynamic_assets", derive(serde::Deserialize))]
pub enum DynamicAsset {
    /// A dynamic asset directly loaded from a single file
    #[cfg_attr(feature = "dynamic_assets", serde(alias = "Folder"))]
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
    /// A dynamic texture atlas asset loaded from a sprite sheet
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
        padding_x: Option<f32>,
        /// Padding between rows in pixels
        padding_y: Option<f32>,
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
pub(crate) fn load_dynamic_asset_collections<S: StateData>(world: &mut World) {
    let cell = world.cell();
    let mut asset_loader_config = cell
        .get_resource_mut::<AssetLoaderConfiguration<S>>()
        .unwrap();
    let asset_server = cell.get_resource::<AssetServer>().unwrap();
    let state = cell
        .get_resource::<State<S>>()
        .expect("Cannot get State resource");
    let mut loading_state = cell
        .get_resource_mut::<State<LoadingState>>()
        .expect("Cannot get LoadingStatePhase");

    let files = asset_loader_config.get_asset_collection_files(state.current());
    if files.is_empty() {
        loading_state.set(LoadingState::LoadingAssets);
        return;
    }
    for file in files {
        asset_loader_config
            .asset_collection_handles
            .push(asset_server.load(&file));
    }
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "2df82c01-9c71-4aa8-adc4-71c5824768f1"]
#[cfg(feature = "dynamic_assets")]
pub struct DynamicAssetCollection(pub(crate) HashMap<String, DynamicAsset>);

#[cfg(feature = "dynamic_assets")]
pub(crate) fn check_dynamic_asset_collections<S: StateData>(world: &mut World) {
    {
        let cell = world.cell();
        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Cannot get AssetServer resource");
        let mut asset_loader_configuration = cell
            .get_resource_mut::<AssetLoaderConfiguration<S>>()
            .expect("Cannot get AssetLoaderConfiguration");
        let mut loading_state = cell
            .get_resource_mut::<State<LoadingState>>()
            .expect("Failed to get loading state");
        let collections_load_state = asset_server.get_group_load_state(
            asset_loader_configuration
                .asset_collection_handles
                .iter()
                .map(|handle| handle.id),
        );
        if collections_load_state == LoadState::Loaded {
            let mut dynamic_asset_collections = cell
                .get_resource_mut::<Assets<DynamicAssetCollection>>()
                .expect("Cannot get AssetServer resource");

            let mut asset_keys = cell.get_resource_mut::<DynamicAssets>().unwrap();
            for collection in asset_loader_configuration
                .asset_collection_handles
                .drain(..)
            {
                let collection = dynamic_asset_collections.remove(collection).unwrap();
                asset_keys.register_dynamic_collection(collection);
            }
            loading_state
                .set(LoadingState::LoadingAssets)
                .expect("Failed to set loading state");
        }
    }
}
