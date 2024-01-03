use crate::dynamic_asset::{DynamicAsset, DynamicAssetType};
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
use bevy::asset::{Asset, AssetServer, Assets, LoadedFolder, UntypedHandle};
use bevy::ecs::system::Command;
use bevy::ecs::world::World;
use bevy::reflect::TypePath;
use bevy::utils::HashMap;
use serde::Deserialize;

#[cfg(feature = "2d")]
use bevy::math::Vec2;
#[cfg(feature = "3d")]
use bevy::pbr::StandardMaterial;
#[cfg(feature = "2d")]
use bevy::sprite::TextureAtlas;

#[cfg(any(feature = "3d", feature = "2d"))]
use bevy::render::texture::{Image, ImageSampler, ImageSamplerDescriptor};
#[cfg(any(feature = "3d", feature = "2d"))]
use serde::Deserializer;

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// An image asset
    #[cfg(any(feature = "3d", feature = "2d"))]
    Image {
        /// Image file path
        path: String,
        /// Sampler
        #[serde(deserialize_with = "deserialize_some", default)]
        sampler: Option<ImageSamplerType>,
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
        /// Sampler
        #[serde(deserialize_with = "deserialize_some", default)]
        sampler: Option<ImageSamplerType>,
        /// The image width in pixels
        tile_size_x: f32,
        /// The image height in pixels
        tile_size_y: f32,
        /// Columns on the sprite sheet
        columns: usize,
        /// Rows on the sprite sheet
        rows: usize,
        /// Padding between columns in pixels
        #[serde(deserialize_with = "deserialize_some", default)]
        padding_x: Option<f32>,
        /// Padding between rows in pixels
        #[serde(deserialize_with = "deserialize_some", default)]
        padding_y: Option<f32>,
        /// Number of pixels offset of the first tile
        #[serde(deserialize_with = "deserialize_some", default)]
        offset_x: Option<f32>,
        /// Number of pixels offset of the first tile
        #[serde(deserialize_with = "deserialize_some", default)]
        offset_y: Option<f32>,
    },
}

#[cfg(any(feature = "3d", feature = "2d"))]
fn deserialize_some<'de, D, G>(deserializer: D) -> Result<Option<G>, D::Error>
where
    D: Deserializer<'de>,
    G: Deserialize<'de>,
{
    let opt: G = G::deserialize(deserializer)?;
    Ok(Some(opt))
}

/// Define the image sampler to configure for an image asset
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ImageSamplerType {
    /// See [`ImageSampler::nearest`]
    Nearest,
    /// See [`ImageSampler::linear`]
    Linear,
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl From<ImageSamplerType> for ImageSamplerDescriptor {
    fn from(value: ImageSamplerType) -> Self {
        match value {
            ImageSamplerType::Nearest => ImageSamplerDescriptor::nearest(),
            ImageSamplerType::Linear => ImageSamplerDescriptor::linear(),
        }
    }
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl From<ImageSamplerType> for ImageSampler {
    fn from(value: ImageSamplerType) -> Self {
        match value {
            ImageSamplerType::Nearest => ImageSampler::nearest(),
            ImageSamplerType::Linear => ImageSampler::linear(),
        }
    }
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
            #[cfg(any(feature = "3d", feature = "2d"))]
            StandardDynamicAsset::Image { path, .. } => {
                vec![asset_server.load::<Image>(path).untyped()]
            }
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
            #[cfg(any(feature = "3d", feature = "2d"))]
            StandardDynamicAsset::Image { path, sampler } => {
                let mut handle = asset_server.load(path);
                if let Some(sampler) = sampler {
                    let mut images = cell
                        .get_resource_mut::<Assets<Image>>()
                        .expect("Cannot get resource Assets<Image>");
                    Self::update_image_sampler(&mut handle, &mut images, sampler);
                }

                Ok(DynamicAssetType::Single(handle.untyped()))
            }
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
                sampler,
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
                let mut handle = asset_server.get_handle(path).unwrap();
                if let Some(sampler_type) = sampler {
                    let mut images = cell
                        .get_resource_mut::<Assets<Image>>()
                        .expect("Cannot get resource Assets<Image>");
                    Self::update_image_sampler(&mut handle, &mut images, sampler_type);
                }
                let texture_atlas_handle = atlases
                    .add(TextureAtlas::from_grid(
                        handle,
                        Vec2::new(*tile_size_x, *tile_size_y),
                        *columns,
                        *rows,
                        Some(Vec2::new(padding_x.unwrap_or(0.), padding_y.unwrap_or(0.))),
                        Some(Vec2::new(offset_x.unwrap_or(0.), offset_y.unwrap_or(0.))),
                    ))
                    .untyped();

                Ok(DynamicAssetType::Single(texture_atlas_handle))
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

#[cfg(any(feature = "3d", feature = "2d"))]
impl StandardDynamicAsset {
    fn update_image_sampler(
        handle: &mut bevy::asset::Handle<Image>,
        images: &mut Assets<Image>,
        sampler_type: &ImageSamplerType,
    ) {
        let image = images.get_mut(&*handle).unwrap();
        let is_different_sampler = if let ImageSampler::Descriptor(descriptor) = &image.sampler {
            let configured_descriptor: ImageSamplerDescriptor = sampler_type.clone().into();
            !descriptor.as_wgpu().eq(&configured_descriptor.as_wgpu())
        } else {
            false
        };

        if is_different_sampler {
            let mut cloned_image = image.clone();
            cloned_image.sampler = sampler_type.clone().into();
            *handle = images.add(cloned_image);
        } else {
            image.sampler = sampler_type.clone().into();
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
/// and combined in [`DynamicAssets`].
#[derive(serde::Deserialize, Asset, TypePath)]
pub struct StandardDynamicAssetCollection(pub HashMap<String, StandardDynamicAsset>);

impl DynamicAssetCollection for StandardDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
