use crate::dynamic_asset::{DynamicAsset, DynamicAssetType};
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
use bevy::asset::{Asset, AssetServer, Assets, LoadedFolder, UntypedHandle};
use bevy::ecs::system::Command;
use bevy::ecs::world::World;
use bevy::reflect::TypePath;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "2d")]
use bevy::math::Vec2;
#[cfg(feature = "3d")]
use bevy::pbr::StandardMaterial;
#[cfg(feature = "2d")]
use bevy::sprite::TextureAtlasLayout;

#[cfg(any(feature = "3d", feature = "2d"))]
use bevy::render::texture::{Image, ImageSampler, ImageSamplerDescriptor};
use bevy::render::texture::{ImageAddressMode, ImageCompareFunction, ImageFilterMode, ImageSamplerBorderColor};

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
        /// ImageAddressMode
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    address_mode_u: Option<ImageAddressModeType>,
        /// ImageAddressMode
	    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    address_mode_v: Option<ImageAddressModeType>,
        /// ImageAddressMode
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    address_mode_w: Option<ImageAddressModeType>,
        /// ImageFilterMode
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    mag_filter: Option<ImageFilterModeType>,
        /// ImageFilterMode
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    min_filter: Option<ImageFilterModeType>,
        /// ImageFilterMode
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    mipmap_filter: Option<ImageFilterModeType>,
        /// f32
	    #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    lod_min_clamp: Option<f32>,
        /// f32
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    lod_max_clamp: Option<f32>,
        /// ImageCompareFunction
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    compare: Option<ImageCompareFunctionType>,
        /// u16
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    anisotropy_clamp: Option<u16>,
        /// ImageSamplerBorderColor
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
	    border_color: Option<ImageSamplerBorderColorType>,
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
        tile_size_x: f32,
        /// The image height in pixels
        tile_size_y: f32,
        /// Columns on the sprite sheet
        columns: usize,
        /// Rows on the sprite sheet
        rows: usize,
        /// Padding between columns in pixels
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        padding_x: Option<f32>,
        /// Padding between rows in pixels
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        padding_y: Option<f32>,
        /// Number of pixels offset of the first tile
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        offset_x: Option<f32>,
        /// Number of pixels offset of the first tile
        #[serde(with = "optional", skip_serializing_if = "Option::is_none", default)]
        offset_y: Option<f32>,
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

/// Defines the ImageAddressMode
#[allow(dead_code)]
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, PartialEq, Clone, Copy, Default, Deserialize, Serialize)]
pub enum ImageAddressModeType {
    #[default]
    /// See [`ImageAddressMode::ClampToEdge`]
    ClampToEdge,
    /// See [`ImageAddressMode::Repeat`]
    Repeat,
    /// See [`ImageAddressMode::MirrorRepeat`]
    MirrorRepeat,
    /// See [`ImageAddressMode::ClampToBorder`]
    ClampToBorder,
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl Into<ImageAddressMode> for ImageAddressModeType {
    fn into(self) -> ImageAddressMode {
        match self {
            ImageAddressModeType::ClampToEdge => ImageAddressMode::ClampToEdge,
            ImageAddressModeType::Repeat => ImageAddressMode::Repeat,
            ImageAddressModeType::MirrorRepeat => ImageAddressMode::MirrorRepeat,
            ImageAddressModeType::ClampToBorder => ImageAddressMode::ClampToBorder,
        }
    }
}

/// Defines the ImageFilterMode
#[allow(dead_code)]
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, PartialEq, Clone, Copy, Default, Deserialize, Serialize)]
pub enum ImageFilterModeType {
    #[default]
    /// See [`ImageFilterMode::Nearest`]
    Nearest,
    /// See [`ImageFilterMode::Linear`]
    Linear,
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl Into<ImageFilterMode> for ImageFilterModeType {
    fn into(self) -> ImageFilterMode {
        match self {
            ImageFilterModeType::Nearest => ImageFilterMode::Nearest,
            ImageFilterModeType::Linear => ImageFilterMode::Linear,
        }
    }
}

/// Defines the ImageCompareFunction
#[allow(dead_code)]
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum ImageCompareFunctionType {
    /// See [`ImageCompareFunction::Never`]
    Never,
    /// See [`ImageCompareFunction::Less`]
    Less,
    /// See [`ImageCompareFunction::Equal`]
    Equal,
    /// See [`ImageCompareFunction::LessEqual`]
    LessEqual,
    /// See [`ImageCompareFunction::Greater`]
    Greater,
    /// See [`ImageCompareFunction::NotEqual`]
    NotEqual,
    /// See [`ImageCompareFunction::GreaterEqual`]
    GreaterEqual,
    /// See [`ImageCompareFunction::Always`]
    Always,
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl Into<ImageCompareFunction> for ImageCompareFunctionType {
    fn into(self) -> ImageCompareFunction {
        match self {
            ImageCompareFunctionType::Never => ImageCompareFunction::Never,
            ImageCompareFunctionType::Less => ImageCompareFunction::Less,
            ImageCompareFunctionType::Equal => ImageCompareFunction::Equal,
            ImageCompareFunctionType::LessEqual => ImageCompareFunction::LessEqual,
            ImageCompareFunctionType::Greater => ImageCompareFunction::Greater,
            ImageCompareFunctionType::NotEqual => ImageCompareFunction::NotEqual,
            ImageCompareFunctionType::GreaterEqual => ImageCompareFunction::GreaterEqual,
            ImageCompareFunctionType::Always => ImageCompareFunction::Always,
        }
    }
}

/// Defines the ImageSamplerBorderColor
#[allow(dead_code)]
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum ImageSamplerBorderColorType {
    /// See [`ImageSamplerBorderColor::TransparentBlack`]
    TransparentBlack,
    /// See [`ImageSamplerBorderColor::OpaqueBlack`]
    OpaqueBlack,
    /// See [`ImageSamplerBorderColor::OpaqueWhite`]
    OpaqueWhite,
    /// See [`ImageSamplerBorderColor::Zero`]
    Zero,
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl Into<ImageSamplerBorderColor> for ImageSamplerBorderColorType {
    fn into(self) -> ImageSamplerBorderColor {
        match self {
            ImageSamplerBorderColorType::TransparentBlack => ImageSamplerBorderColor::TransparentBlack,
            ImageSamplerBorderColorType::OpaqueBlack => ImageSamplerBorderColor::OpaqueBlack,
            ImageSamplerBorderColorType::OpaqueWhite => ImageSamplerBorderColor::OpaqueWhite,
            ImageSamplerBorderColorType::Zero => ImageSamplerBorderColor::Zero,
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
        let cell = world.cell();
        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Cannot get AssetServer");
        match self {
            StandardDynamicAsset::File { path } => Ok(DynamicAssetType::Single(
                asset_server.get_handle_untyped(path).unwrap(),
            )),
            #[cfg(any(feature = "3d", feature = "2d"))]
            StandardDynamicAsset::Image {
                path,
                sampler,
                address_mode_u,
                address_mode_v,
                address_mode_w,
                mag_filter,
                min_filter,
                mipmap_filter,
                lod_min_clamp,
                lod_max_clamp,
                compare,
                anisotropy_clamp,
                border_color,
            } => {
                let mut handle = asset_server.load(path);
                if sampler.is_some()
                    || address_mode_u.is_some()
                    || address_mode_v.is_some()
                    || address_mode_w.is_some()
                    || mag_filter.is_some()
                    || min_filter.is_some()
                    || mipmap_filter.is_some()
                    || lod_min_clamp.is_some()
                    || lod_max_clamp.is_some()
                    || compare.is_some()
                    || anisotropy_clamp.is_some()
                    || border_color.is_some()
                {
                    let mut images = cell
                        .get_resource_mut::<Assets<Image>>()
                        .expect("Cannot get resource Assets<Image>");
                    Self::update_image_sampler(
                        &mut handle, &mut images,
                        sampler,
                        address_mode_u,
                        address_mode_v,
                        address_mode_w,
                        mag_filter,
                        min_filter,
                        mipmap_filter,
                        lod_min_clamp,
                        lod_max_clamp,
                        compare,
                        anisotropy_clamp,
                        border_color,
                    );
                }

                Ok(DynamicAssetType::Single(handle.untyped()))
            }
            #[cfg(feature = "3d")]
            StandardDynamicAsset::StandardMaterial { path } => {
                let mut materials = cell
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .expect("Cannot get resource Assets<StandardMaterial>");
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
                let mut atlases = cell
                    .get_resource_mut::<Assets<TextureAtlasLayout>>()
                    .expect("Cannot get resource Assets<TextureAtlasLayout>");
                let texture_atlas_handle = atlases
                    .add(TextureAtlasLayout::from_grid(
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
        sampler_type: &Option<ImageSamplerType>,
        address_mode_u: &Option<ImageAddressModeType>,
        address_mode_v: &Option<ImageAddressModeType>,
        address_mode_w: &Option<ImageAddressModeType>,
        mag_filter: &Option<ImageFilterModeType>,
        min_filter: &Option<ImageFilterModeType>,
        mipmap_filter: &Option<ImageFilterModeType>,
        lod_min_clamp: &Option<f32>,
        lod_max_clamp: &Option<f32>,
        compare: &Option<ImageCompareFunctionType>,
        anisotropy_clamp: &Option<u16>,
        border_color: &Option<ImageSamplerBorderColorType>,
    ) {
        let image = images.get_mut(&*handle).unwrap();
        
        let mut descriptor = ImageSamplerDescriptor::default();
        descriptor.address_mode_u = address_mode_u.unwrap_or_default().into();
        descriptor.address_mode_v = address_mode_v.unwrap_or_default().into();
        descriptor.address_mode_w = address_mode_w.unwrap_or_default().into();
        if let Some(sampler) = sampler_type {
            match sampler {
                ImageSamplerType::Linear => {
                    descriptor.mag_filter = ImageFilterModeType::Linear.into();
                    descriptor.min_filter = ImageFilterModeType::Linear.into();
                    descriptor.mipmap_filter = ImageFilterModeType::Linear.into();
                }
                ImageSamplerType::Nearest => {
                    descriptor.mag_filter = ImageFilterModeType::Nearest.into();
                    descriptor.min_filter = ImageFilterModeType::Nearest.into();
                    descriptor.mipmap_filter = ImageFilterModeType::Nearest.into();
                }
            }
        } else {
            descriptor.mag_filter = mag_filter.unwrap_or_default().into();
            descriptor.min_filter = min_filter.unwrap_or_default().into();
            descriptor.mipmap_filter = mipmap_filter.unwrap_or_default().into();
        }
        descriptor.lod_min_clamp = lod_min_clamp.unwrap_or_default().into();
        descriptor.lod_max_clamp = lod_max_clamp.unwrap_or_default().into();
        descriptor.compare = if let Some(compare) = compare { Some(compare.clone().into()) } else { None };
        descriptor.anisotropy_clamp = anisotropy_clamp.unwrap_or_default().into();
        descriptor.border_color = if let Some(border_color) = border_color { Some(border_color.clone().into()) } else { None };
        
        let is_different_sampler = if let ImageSampler::Descriptor(original_descriptor) = &image.sampler {
            !original_descriptor.as_wgpu().eq(&descriptor.as_wgpu())
        } else {
            false
        };
        
        if is_different_sampler {
            let mut cloned_image = image.clone();
            cloned_image.sampler = ImageSampler::Descriptor(descriptor);
            *handle = images.add(cloned_image);
        } else {
            image.sampler = ImageSampler::Descriptor(descriptor);
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
        tile_size_x: 96.0,
        tile_size_y: 99.0,
        columns: 8,
        rows: 1,
        padding_x: 42.42,
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
        address_mode_u: Repeat,
        compare: Less,
    ),
})"#;
        serialize_and_deserialize(dynamic_asset_file);
    }

    #[test]
    fn serialize_and_deserialize_array() {
        let dynamic_asset_file = r#"({
    "layouts": [
        TextureAtlasLayout(
            tile_size_x: 32.0,
            tile_size_y: 32.0,
            columns: 12,
            rows: 12,
        ),
        TextureAtlasLayout(
            tile_size_x: 32.0,
            tile_size_y: 64.0,
            columns: 12,
            rows: 6,
        ),
        TextureAtlasLayout(
            tile_size_x: 64.0,
            tile_size_y: 32.0,
            columns: 6,
            rows: 12,
        ),
        TextureAtlasLayout(
            tile_size_x: 64.0,
            tile_size_y: 64.0,
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
