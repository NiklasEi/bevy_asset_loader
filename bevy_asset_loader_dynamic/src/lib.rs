//! This crate adds support for auto deriving [bevy_asset_loader::AssetCollection]
//!
//! You do not have to use it directly. Just import ``AssetCollection`` from ``bevy_asset_loader``
//! and use ``#[derive(AssetCollection)]`` to derive the trait.

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
pub enum DynamicAsset {
    File {
        path: String,
    },
    #[cfg(feature = "render")]
    StandardMaterial {
        path: String,
    },
    #[cfg(feature = "render")]
    TextureAtlas {
        path: String,
        tile_size_x: f32,
        tile_size_y: f32,
        columns: usize,
        rows: usize,
        padding_x: f32,
        padding_y: f32,
    },
}

impl DynamicAsset {
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
