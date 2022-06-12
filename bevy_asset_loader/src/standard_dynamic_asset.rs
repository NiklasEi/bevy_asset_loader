use crate::dynamic_asset::{DynamicAsset, DynamicAssetType};
use bevy::asset::{AssetServer, HandleUntyped};
use bevy::ecs::world::World;

#[cfg(feature = "dynamic_assets")]
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
#[cfg(feature = "dynamic_assets")]
use bevy::reflect::TypeUuid;
#[cfg(feature = "dynamic_assets")]
use bevy::utils::HashMap;

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "dynamic_assets", derive(serde::Deserialize))]
pub enum StandardDynamicAsset {
    /// A dynamic asset directly loaded from a single file
    File {
        /// Asset file path
        path: String,
    },
    /// A folder to load all including asset files from
    ///
    /// Subdirectories are also included.
    /// This is not supported for web builds! If you need compatibility with web builds,
    /// consider using [`DynamicAsset::Files`] instead.
    Folder {
        /// Asset file folder path
        path: String,
    },
    /// A list of files to be loaded as a vector of handles
    Files {
        /// Asset file paths
        paths: Vec<String>,
    },
    /// A dynamic standard material asset directly loaded from an image file
    #[cfg(feature = "3d")]
    StandardMaterial {
        /// Asset file path
        path: String,
    },
    /// A dynamic texture atlas asset loaded from a sprite sheet
    #[cfg(feature = "2d")]
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

impl DynamicAsset for StandardDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped> {
        match self {
            StandardDynamicAsset::File { path } => vec![asset_server.load_untyped(path)],
            StandardDynamicAsset::Folder { path } => asset_server
                .load_folder(path)
                .unwrap_or_else(|_| panic!("Failed to load '{}' as a folder", path)),
            StandardDynamicAsset::Files { paths } => paths
                .iter()
                .map(|path| asset_server.load_untyped(path))
                .collect(),
            #[cfg(feature = "3d")]
            StandardDynamicAsset::StandardMaterial { path } => {
                vec![asset_server.load_untyped(path)]
            }
            #[cfg(feature = "2d")]
            StandardDynamicAsset::TextureAtlas { path, .. } => {
                vec![asset_server.load_untyped(path)]
            }
        }
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        let cell = world.cell();
        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Cannot get AssetServer");
        match self {
            StandardDynamicAsset::File { path } => Ok(DynamicAssetType::Single(
                asset_server.get_handle_untyped(path),
            )),
            #[cfg(feature = "3d")]
            StandardDynamicAsset::StandardMaterial { path } => {
                let mut materials = cell
                    .get_resource_mut::<bevy::asset::Assets<bevy::pbr::StandardMaterial>>()
                    .expect("Cannot get resource Assets<StandardMaterial>");
                let handle = materials
                    .add(
                        asset_server
                            .get_handle::<bevy::render::texture::Image, &String>(path)
                            .into(),
                    )
                    .clone_untyped();

                Ok(DynamicAssetType::Single(handle))
            }
            #[cfg(feature = "2d")]
            StandardDynamicAsset::TextureAtlas {
                path,
                tile_size_x,
                tile_size_y,
                columns,
                rows,
                padding_x,
                padding_y,
            } => {
                let mut atlases = cell
                    .get_resource_mut::<bevy::asset::Assets<bevy::sprite::TextureAtlas>>()
                    .expect("Cannot get resource Assets<TextureAtlas>");
                let handle = atlases
                    .add(bevy::sprite::TextureAtlas::from_grid_with_padding(
                        asset_server.get_handle(path),
                        bevy::math::Vec2::new(*tile_size_x, *tile_size_y),
                        *columns,
                        *rows,
                        bevy::math::Vec2::new(padding_x.unwrap_or(0.), padding_y.unwrap_or(0.)),
                    ))
                    .clone_untyped();

                Ok(DynamicAssetType::Single(handle))
            }
            StandardDynamicAsset::Folder { path } => Ok(DynamicAssetType::Collection(
                asset_server
                    .load_folder(path)
                    .unwrap_or_else(|_| panic!("Failed to load '{}' as a folder", path)),
            )),
            StandardDynamicAsset::Files { paths } => Ok(DynamicAssetType::Collection(
                paths
                    .iter()
                    .map(|path| asset_server.load_untyped(path))
                    .collect(),
            )),
        }
    }
}

/// The asset defining a mapping from asset keys to dynamic assets
///
/// These assets are loaded at the beginning of a loading state
/// and combined in [`DynamicAssets`](DynamicAssets).
#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "2df82c01-9c71-4aa8-adc4-71c5824768f1"]
pub struct StandardDynamicAssetCollection(pub HashMap<String, StandardDynamicAsset>);

impl DynamicAssetCollection for StandardDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
