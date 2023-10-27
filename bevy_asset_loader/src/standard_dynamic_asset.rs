use crate::dynamic_asset::{DynamicAsset, DynamicAssetType};
use bevy::asset::{Asset, AssetServer, Assets, LoadedFolder, UntypedHandle};
use bevy::ecs::system::Command;
use bevy::ecs::world::World;
#[cfg(feature = "2d")]
use bevy::math::Vec2;
#[cfg(feature = "3d")]
use bevy::pbr::StandardMaterial;
#[cfg(any(feature = "2d", feature = "3d"))]
use bevy::render::texture::Image;
#[cfg(feature = "2d")]
use bevy::sprite::TextureAtlas;

use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
use bevy::reflect::TypePath;
use bevy::utils::HashMap;

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
#[derive(Debug, Clone, serde::Deserialize)]
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
    /// consider using [`StandardDynamicAsset::Files`] instead.
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
        /// Number of pixels offset of the first tile
        offset_x: Option<f32>,
        /// Number of pixels offset of the first tile
        offset_y: Option<f32>,
    },
}

impl DynamicAsset for StandardDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            StandardDynamicAsset::File { path } => vec![asset_server.load_untyped(path).untyped()],
            StandardDynamicAsset::Folder { path } => vec![asset_server.load_folder(path).untyped()],
            StandardDynamicAsset::Files { paths } => paths
                .iter()
                .map(|path| asset_server.load_untyped(path).untyped())
                .collect(),
            #[cfg(feature = "3d")]
            StandardDynamicAsset::StandardMaterial { path } => {
                vec![asset_server.load::<Image>(path).untyped()]
            }
            #[cfg(feature = "2d")]
            StandardDynamicAsset::TextureAtlas { path, .. } => {
                vec![asset_server.load::<Image>(path).untyped()]
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
                asset_server.get_handle_untyped(path).unwrap(),
            )),
            #[cfg(feature = "3d")]
            StandardDynamicAsset::StandardMaterial { path } => {
                let mut materials = cell
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .expect("Cannot get resource Assets<StandardMaterial>");
                let handle = materials
                    .add(asset_server.get_handle::<Image>(path).unwrap().into())
                    .untyped();

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
                offset_x,
                offset_y,
            } => {
                let mut atlases = cell
                    .get_resource_mut::<Assets<TextureAtlas>>()
                    .expect("Cannot get resource Assets<TextureAtlas>");
                let handle = atlases
                    .add(TextureAtlas::from_grid(
                        asset_server.get_handle(path).unwrap(),
                        Vec2::new(*tile_size_x, *tile_size_y),
                        *columns,
                        *rows,
                        Some(Vec2::new(padding_x.unwrap_or(0.), padding_y.unwrap_or(0.))),
                        Some(Vec2::new(offset_x.unwrap_or(0.), offset_y.unwrap_or(0.))),
                    ))
                    .untyped();

                Ok(DynamicAssetType::Single(handle))
            }
            StandardDynamicAsset::Folder { path } => {
                let folders = cell
                    .get_resource_mut::<Assets<LoadedFolder>>()
                    .expect("Cannot get resource Assets<LoadedFolder>");
                Ok(DynamicAssetType::Collection(
                    folders
                        .get(asset_server.get_handle(path).unwrap())
                        .unwrap()
                        .handles
                        .to_vec(),
                ))
            }
            StandardDynamicAsset::Files { paths } => Ok(DynamicAssetType::Collection(
                paths
                    .iter()
                    .map(|path| {
                        asset_server
                            .get_handle_untyped(path)
                            .expect("No Handle for path")
                    })
                    .collect(),
            )),
        }
    }
}

/// Command to register a standard dynamic asset under the given key
pub struct RegisterStandardDynamicAsset<K: Into<String> + Sync + Send + 'static> {
    /// The key of the asset
    pub key: K,
    /// The dynamic asset to be registered
    pub asset: StandardDynamicAsset,
}

impl<K: Into<String> + Sync + Send + 'static> Command for RegisterStandardDynamicAsset<K> {
    fn apply(self, world: &mut World) {
        let mut dynamic_assets = world.resource_mut::<DynamicAssets>();
        dynamic_assets.register_asset(self.key, Box::new(self.asset));
    }
}

/// The asset defining a mapping from asset keys to dynamic assets
///
/// These assets are loaded at the beginning of a loading state
/// and combined in [`DynamicAssets`](DynamicAssets).
#[derive(serde::Deserialize, Asset, TypePath)]
pub struct StandardDynamicAssetCollection(pub HashMap<String, StandardDynamicAsset>);

impl DynamicAssetCollection for StandardDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
