use crate::dynamic_asset::{DynamicAsset, DynamicAssetType};
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
use bevy_asset::{Asset, AssetServer, Assets, LoadedFolder, UntypedHandle};
use bevy_ecs::system::SystemState;
use bevy_ecs::world::{Command, World};
use bevy_reflect::TypePath;
use bevy_utils::HashMap;
use serde::{Deserialize, Serialize};

use bevy_ecs::system::{Res, ResMut};
#[cfg(feature = "2d")]
use bevy_math::UVec2;
#[cfg(feature = "3d")]
use bevy_pbr::StandardMaterial;
#[cfg(feature = "2d")]
use bevy_sprite::TextureAtlasLayout;

#[cfg(any(feature = "3d", feature = "2d"))]
use bevy_render::texture::{Image, ImageSampler, ImageSamplerDescriptor};

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        sampler: Option<ImageSamplerType>,
        /// array texture layers
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        array_texture_layers: Option<u32>,
    },
    /// A dynamic standard material asset directly loaded from an image file
    #[cfg(feature = "3d")]
    StandardMaterial {
        /// Asset file path
        path: String,
    },
    /// A dynamic texture atlas asset loaded from a sprite sheet
    #[cfg(feature = "2d")]
    TextureAtlasLayout {
        /// The image width in pixels
        tile_size_x: u32,
        /// The image height in pixels
        tile_size_y: u32,
        /// Columns on the sprite sheet
        columns: u32,
        /// Rows on the sprite sheet
        rows: u32,
        /// Padding between columns in pixels
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        padding_x: Option<u32>,
        /// Padding between rows in pixels
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        padding_y: Option<u32>,
        /// Number of pixels offset of the first tile
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        offset_x: Option<u32>,
        /// Number of pixels offset of the first tile
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        offset_y: Option<u32>,
    },
}

#[cfg(any(feature = "3d", feature = "2d"))]
mod optional {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn deserialize<'de, D, G>(deserializer: D) -> Result<Option<G>, D::Error>
    where
        D: Deserializer<'de>,
        G: Deserialize<'de>,
    {
        let opt: G = G::deserialize(deserializer)?;
        Ok(Some(opt))
    }

    pub(super) fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize + std::fmt::Debug,
        S: Serializer,
    {
        match value.as_ref() {
            Some(value) => value.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

/// Define the image sampler to configure for an image asset
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
            StandardDynamicAsset::TextureAtlasLayout { .. } => {
                vec![]
            }
        }
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            StandardDynamicAsset::File { path } => {
                let asset_server = world
                    .get_resource::<AssetServer>()
                    .expect("Cannot get AssetServer");
                Ok(DynamicAssetType::Single(
                    asset_server.get_handle_untyped(path).unwrap(),
                ))
            }
            #[cfg(any(feature = "3d", feature = "2d"))]
            StandardDynamicAsset::Image {
                path,
                sampler,
                array_texture_layers,
            } => {
                let mut system_state =
                    SystemState::<(ResMut<Assets<Image>>, Res<AssetServer>)>::new(world);
                let (mut images, asset_server) = system_state.get_mut(world);
                let mut handle = asset_server.load(path);
                if let Some(sampler) = sampler {
                    Self::update_image_sampler(&mut handle, &mut images, sampler);
                }
                if let Some(layers) = array_texture_layers {
                    let image = images
                        .get_mut(&handle)
                        .expect("Failed to find loaded image");
                    image.reinterpret_stacked_2d_as_array(*layers);
                }

                Ok(DynamicAssetType::Single(handle.untyped()))
            }
            #[cfg(feature = "3d")]
            StandardDynamicAsset::StandardMaterial { path } => {
                let mut system_state =
                    SystemState::<(ResMut<Assets<StandardMaterial>>, Res<AssetServer>)>::new(world);
                let (mut materials, asset_server) = system_state.get_mut(world);
                let handle = materials
                    .add(StandardMaterial::from(
                        asset_server.get_handle::<Image>(path).unwrap(),
                    ))
                    .untyped();

                Ok(DynamicAssetType::Single(handle))
            }
            #[cfg(feature = "2d")]
            StandardDynamicAsset::TextureAtlasLayout {
                tile_size_x,
                tile_size_y,
                columns,
                rows,
                padding_x,
                padding_y,
                offset_x,
                offset_y,
            } => {
                let mut atlases = world
                    .get_resource_mut::<Assets<TextureAtlasLayout>>()
                    .expect("Cannot get Assets<TextureAtlasLayout>");
                let texture_atlas_handle = atlases
                    .add(TextureAtlasLayout::from_grid(
                        UVec2::new(*tile_size_x, *tile_size_y),
                        *columns,
                        *rows,
                        Some(UVec2::new(padding_x.unwrap_or(0), padding_y.unwrap_or(0))),
                        Some(UVec2::new(offset_x.unwrap_or(0), offset_y.unwrap_or(0))),
                    ))
                    .untyped();

                Ok(DynamicAssetType::Single(texture_atlas_handle))
            }
            StandardDynamicAsset::Folder { path } => {
                let mut system_state =
                    SystemState::<(Res<Assets<LoadedFolder>>, Res<AssetServer>)>::new(world);
                let (folders, asset_server) = system_state.get(world);
                Ok(DynamicAssetType::Collection(
                    folders
                        .get(&asset_server.get_handle(path).unwrap())
                        .unwrap()
                        .handles
                        .to_vec(),
                ))
            }
            StandardDynamicAsset::Files { paths } => {
                let asset_server = world
                    .get_resource::<AssetServer>()
                    .expect("Cannot get AssetServer");
                Ok(DynamicAssetType::Collection(
                    paths
                        .iter()
                        .map(|path| {
                            asset_server
                                .get_handle_untyped(path)
                                .expect("No Handle for path")
                        })
                        .collect(),
                ))
            }
        }
    }
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl StandardDynamicAsset {
    fn update_image_sampler(
        handle: &mut bevy_asset::Handle<Image>,
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
#[derive(Deserialize, Serialize, Asset, TypePath, PartialEq, Debug)]
pub struct StandardDynamicAssetCollection(pub HashMap<String, StandardDynamicAsset>);

impl DynamicAssetCollection for StandardDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}

impl DynamicAsset for Vec<StandardDynamicAsset> {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        self.iter()
            .flat_map(|asset| asset.load(asset_server))
            .collect()
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        let mut all_handles = vec![];

        for asset in self {
            match asset.build(world)? {
                DynamicAssetType::Single(handle) => all_handles.push(handle),
                DynamicAssetType::Collection(handles) => all_handles.extend(handles),
            }
        }

        Ok(DynamicAssetType::Collection(all_handles))
    }
}

/// The asset defining a mapping from asset keys to an array of dynamic assets.
///
/// These assets are loaded at the beginning of a loading state
/// and combined in [`DynamicAssets`].
///
/// Example:
/// ```ron
/// ({
///     "layouts": [
///         TextureAtlasLayout(
///             tile_size_x: 32.,
///             tile_size_y: 32.,
///             columns: 12,
///             rows: 12,
///         ),
///         TextureAtlasLayout(
///             tile_size_x: 32.,
///             tile_size_y: 64.,
///             columns: 12,
///             rows: 6,
///         ),
///         TextureAtlasLayout(
///             tile_size_x: 64.,
///             tile_size_y: 32.,
///             columns: 6,
///             rows: 12,
///         ),
///         TextureAtlasLayout(
///             tile_size_x: 64.,
///             tile_size_y: 64.,
///             columns: 6,
///             rows: 6,
///         ),
///     ],
///     "mixed": [
///         StandardMaterial(
///             path: "images/tree.png",
///         ),
///         Image(
///             path: "ryot_mascot.png",
///             sampler: Nearest,
///         ),
///         Image(
///             path: "ryot_mascot.png",
///             sampler: Nearest,
///         ),
///     ],
/// })
/// ```
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_asset_loader::prelude::*;
///
/// #[derive(AssetCollection, Resource)]
/// struct MyAssets {
///     #[asset(key = "layouts", collection(typed))]
///     atlas_layout: Vec<Handle<TextureAtlasLayout>>,
///
///     #[asset(key = "mixed", collection)]
///     mixed_handlers: Vec<UntypedHandle>,
/// }
/// ```
#[derive(Deserialize, Serialize, Asset, TypePath, PartialEq, Debug)]
pub struct StandardDynamicAssetArrayCollection(HashMap<String, Vec<StandardDynamicAsset>>);

impl DynamicAssetCollection for StandardDynamicAssetArrayCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}

#[cfg(test)]
#[cfg(feature = "2d")]
mod tests {
    use crate::prelude::StandardDynamicAssetCollection;
    use crate::standard_dynamic_asset::StandardDynamicAssetArrayCollection;

    #[test]
    fn serialize_and_deserialize_atlas() {
        let dynamic_asset_file = r#"({
    "texture_atlas": TextureAtlasLayout(
        tile_size_x: 96,
        tile_size_y: 99,
        columns: 8,
        rows: 1,
        padding_x: 42,
    ),
})"#;
        serialize_and_deserialize(dynamic_asset_file);
    }

    #[test]
    fn serialize_and_deserialize_image() {
        let dynamic_asset_file = r#"({
    "image": Image(
        path: "images/player.png",
        sampler: Linear,
    ),
})"#;
        serialize_and_deserialize(dynamic_asset_file);
    }

    #[test]
    fn serialize_and_deserialize_array() {
        let dynamic_asset_file = r#"({
    "layouts": [
        TextureAtlasLayout(
            tile_size_x: 32,
            tile_size_y: 32,
            columns: 12,
            rows: 12,
        ),
        TextureAtlasLayout(
            tile_size_x: 32,
            tile_size_y: 64,
            columns: 12,
            rows: 6,
        ),
        TextureAtlasLayout(
            tile_size_x: 64,
            tile_size_y: 32,
            columns: 6,
            rows: 12,
        ),
        TextureAtlasLayout(
            tile_size_x: 64,
            tile_size_y: 64,
            columns: 6,
            rows: 6,
        ),
    ],
    "mixed": [
        StandardMaterial(
            path: "images/tree.png",
        ),
        Image(
            path: "ryot_mascot.png",
            sampler: Nearest,
        ),
        Image(
            path: "ryot_mascot.png",
            sampler: Nearest,
        ),
    ],
})"#;

        let before: StandardDynamicAssetArrayCollection =
            ron::from_str(dynamic_asset_file).unwrap();

        let serialized_dynamic_asset_file = ron::ser::to_string_pretty(
            &before,
            ron::ser::PrettyConfig::default().new_line("\n".to_string()),
        )
        .unwrap();

        let after: StandardDynamicAssetArrayCollection =
            ron::from_str(&serialized_dynamic_asset_file).unwrap();

        assert_eq!(before, after);
    }

    fn serialize_and_deserialize(dynamic_asset_file: &'static str) {
        let before: StandardDynamicAssetCollection = ron::from_str(dynamic_asset_file).unwrap();

        let serialized_dynamic_asset_file = ron::ser::to_string_pretty(
            &before,
            ron::ser::PrettyConfig::default().new_line("\n".to_string()),
        )
        .unwrap();
        assert_eq!(dynamic_asset_file, &serialized_dynamic_asset_file);
    }
}
