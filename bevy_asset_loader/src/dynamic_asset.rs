use bevy::asset::AssetServer;
use bevy::ecs::prelude::World;
use bevy::ecs::schedule::StateData;

use crate::{AssetKeys, AssetLoaderConfiguration};

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
pub(crate) fn prepare_asset_keys<State: StateData>(world: &mut World) {
    println!("prepare_asset_keys");
    let cell = world.cell();
    let mut asset_keys = cell.get_resource_mut::<AssetKeys>().unwrap();
    let mut asset_loader_config = cell
        .get_resource_mut::<AssetLoaderConfiguration<State>>()
        .unwrap();
    let asset_server = cell.get_resource::<AssetServer>().unwrap();

    let files = asset_keys.take_asset_files();
    for file in files {
        asset_loader_config
            .asset_keys
            .push(asset_server.load(&file));
    }
}
