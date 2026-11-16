use crate::dynamic_asset::{DynamicAsset, DynamicAssetType};
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
use bevy_app::{App, Plugin};
use bevy_asset::io::Reader;
use bevy_asset::{
    Asset, AssetApp, AssetLoader, AssetPath, AssetServer, Assets, LoadContext, LoadedFolder,
    UntypedHandle,
};
use bevy_ecs::{
    change_detection::Res,
    system::{Command, SystemState},
    world::World,
};
use bevy_platform::collections::HashMap;
use bevy_reflect::TypePath;
use ron::error::{Position, Span, SpannedError, SpannedResult};
use ron::value::RawValue;
use serde::{Deserialize, Serialize};
use std::any::TypeId;

#[cfg(feature = "2d")]
use bevy_image::TextureAtlasLayout;
#[cfg(feature = "2d")]
use bevy_math::UVec2;
#[cfg(feature = "3d")]
use bevy_pbr::StandardMaterial;

#[cfg(any(feature = "3d", feature = "2d"))]
use bevy_ecs::change_detection::ResMut;
#[cfg(any(feature = "3d", feature = "2d"))]
use bevy_image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};

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
        #[serde(default, skip_serializing_if = "is_default")]
        sampler: ImageSamplerType,
        /// Sampler
        #[serde(default, skip_serializing_if = "is_default")]
        wrap: ImageAddressModeType,
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

#[cfg(any(feature = "3d", feature = "2d"))]
fn is_default<T: Default + PartialEq + Copy>(value: &T) -> bool {
    T::default() == *value
}

/// Define the image sampler to configure for an image asset
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
pub enum ImageSamplerType {
    /// See [`ImageSampler::nearest`]
    #[default]
    Nearest,
    /// See [`ImageSampler::linear`]
    Linear,
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl From<&ImageSamplerType> for ImageFilterMode {
    fn from(value: &ImageSamplerType) -> Self {
        match value {
            ImageSamplerType::Nearest => ImageFilterMode::Nearest,
            ImageSamplerType::Linear => ImageFilterMode::Linear,
        }
    }
}

/// Define the image sampler address mode
#[cfg(any(feature = "3d", feature = "2d"))]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
pub enum ImageAddressModeType {
    /// See [`ImageAddressMode::ClampToEdge`]
    #[default]
    ClampToEdge,
    /// See [`ImageAddressMode::Repeat`]
    Repeat,
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl From<&ImageAddressModeType> for ImageAddressMode {
    fn from(value: &ImageAddressModeType) -> Self {
        match value {
            ImageAddressModeType::ClampToEdge => ImageAddressMode::ClampToEdge,
            ImageAddressModeType::Repeat => ImageAddressMode::Repeat,
        }
    }
}

impl DynamicAsset for StandardDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            StandardDynamicAsset::File { path } => {
                vec![asset_server.load_builder().load_untyped(path).untyped()]
            }
            StandardDynamicAsset::Folder { path } => vec![asset_server.load_folder(path).untyped()],
            StandardDynamicAsset::Files { paths } => paths
                .iter()
                .map(|path| asset_server.load_builder().load_untyped(path).untyped())
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
                wrap: address_mode,
                array_texture_layers,
            } => {
                let mut system_state =
                    SystemState::<(ResMut<Assets<Image>>, Res<AssetServer>)>::new(world);
                let (mut images, asset_server) = system_state.get_mut(world)?;
                let mut handle = asset_server.load(path);
                Self::update_image_sampler(&mut handle, &mut images, sampler, address_mode);
                if let Some(layers) = array_texture_layers {
                    let mut image = images
                        .get_mut(&handle)
                        .expect("Failed to find loaded image");
                    let _ = image.reinterpret_stacked_2d_as_array(*layers);
                }

                Ok(DynamicAssetType::Single(handle.untyped()))
            }
            #[cfg(feature = "3d")]
            StandardDynamicAsset::StandardMaterial { path } => {
                let mut system_state =
                    SystemState::<(ResMut<Assets<StandardMaterial>>, Res<AssetServer>)>::new(world);
                let (mut materials, asset_server) = system_state.get_mut(world)?;
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
                let (folders, asset_server) = system_state.get(world)?;
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
    fn load_as(&self, world: &mut World, asset_type: TypeId) -> Vec<UntypedHandle> {
        let asset_server = world.resource::<AssetServer>().clone();
        match self {
            StandardDynamicAsset::File { path } => {
                vec![asset_server.load_builder().load_erased(asset_type, path)]
            }
            StandardDynamicAsset::Files { paths } => paths
                .iter()
                .map(|path| asset_server.load_builder().load_erased(asset_type, path))
                .collect(),
            StandardDynamicAsset::Folder { path } => {
                vec![crate::typed_folder::load_folder_as(world, path, asset_type).untyped()]
            }
            // These variants decide their asset type themselves
            #[allow(unreachable_patterns)]
            _ => self.load(&asset_server),
        }
    }

    fn build_as(
        &self,
        world: &mut World,
        asset_type: TypeId,
    ) -> Result<DynamicAssetType, anyhow::Error> {
        let typed_handle = |asset_server: &AssetServer, path: &String| {
            asset_server
                .get_path_and_type_id_handle(&AssetPath::from(path.clone()), asset_type)
                .ok_or_else(|| anyhow::anyhow!("No handle for '{path}' with the requested asset type. Was it loaded with `load_as`?"))
        };
        match self {
            StandardDynamicAsset::File { path } => {
                let asset_server = world.resource::<AssetServer>();
                Ok(DynamicAssetType::Single(typed_handle(asset_server, path)?))
            }
            StandardDynamicAsset::Files { paths } => {
                let asset_server = world.resource::<AssetServer>();
                Ok(DynamicAssetType::Collection(
                    paths
                        .iter()
                        .map(|path| typed_handle(asset_server, path))
                        .collect::<Result<_, _>>()?,
                ))
            }
            StandardDynamicAsset::Folder { path } => {
                let handle = crate::typed_folder::get_folder_as(world, path, asset_type)
                    .ok_or_else(|| {
                        anyhow::anyhow!("The folder '{path}' was not loaded with `load_as`")
                    })?;
                let folders = world.resource::<Assets<LoadedFolder>>();
                let folder = folders
                    .get(&handle)
                    .ok_or_else(|| anyhow::anyhow!("The folder '{path}' is not loaded"))?;
                Ok(DynamicAssetType::Collection(folder.handles.to_vec()))
            }
            #[allow(unreachable_patterns)]
            _ => self.build(world),
        }
    }
}

#[cfg(any(feature = "3d", feature = "2d"))]
impl StandardDynamicAsset {
    fn update_image_sampler(
        handle: &mut bevy_asset::Handle<Image>,
        images: &mut Assets<Image>,
        sampler_type: &ImageSamplerType,
        address_mode: &ImageAddressModeType,
    ) {
        let mut image = images.get_mut(&*handle).unwrap();
        let configured_descriptor = ImageSamplerDescriptor {
            address_mode_u: address_mode.into(),
            address_mode_v: address_mode.into(),
            address_mode_w: address_mode.into(),
            mag_filter: sampler_type.into(),
            min_filter: sampler_type.into(),
            mipmap_filter: sampler_type.into(),
            ..Default::default()
        };
        let is_different_sampler = if let ImageSampler::Descriptor(descriptor) = &image.sampler {
            !descriptor.as_wgpu().eq(&configured_descriptor.as_wgpu())
        } else {
            false
        };

        if is_different_sampler {
            let mut cloned_image = image.clone();
            cloned_image.sampler = ImageSampler::Descriptor(configured_descriptor);
            drop(image);
            *handle = images.add(cloned_image);
        } else {
            image.sampler = ImageSampler::Descriptor(configured_descriptor);
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
    type Out = ();

    fn apply(self, world: &mut World) {
        let mut dynamic_assets = world.resource_mut::<DynamicAssets>();
        dynamic_assets.register_asset(self.key, Box::new(self.asset));
    }
}

/// The asset defining a mapping from asset keys to dynamic assets
///
/// These assets are loaded at the beginning of a loading state
/// and combined in [`DynamicAssets`].
///
/// Every key maps to a single [`StandardDynamicAsset`], an array of them, or a map of them.
///
/// ```ron
/// ({
///     "player": Image(
///         path: "images/player.png",
///         sampler: Nearest,
///     ),
///     "layouts": [
///         TextureAtlasLayout(
///             tile_size_x: 32,
///             tile_size_y: 32,
///             columns: 12,
///             rows: 12,
///         ),
///         TextureAtlasLayout(
///             tile_size_x: 32,
///             tile_size_y: 64,
///             columns: 12,
///             rows: 6,
///         ),
///     ],
///     "icons": {
///         "play": Image(path: "images/play.png"),
///         "pause": Image(path: "images/pause.png"),
///     },
/// })
/// ```
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy::platform::collections::HashMap;
/// # use bevy_asset_loader::prelude::*;
/// #[derive(AssetCollection, Resource)]
/// struct MyAssets {
///     #[asset(key = "player")]
///     player: Handle<Image>,
///     #[asset(key = "layouts", collection(typed))]
///     layouts: Vec<Handle<TextureAtlasLayout>>,
///     #[asset(key = "icons", collection(mapped, typed))]
///     icons: HashMap<String, Handle<Image>>,
/// }
/// ```
#[derive(Deserialize, Serialize, Asset, TypePath, PartialEq, Debug)]
pub struct StandardDynamicAssetCollection(pub HashMap<String, StandardDynamicAssetEntry>);

impl DynamicAssetCollection for StandardDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, entry) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(entry.clone()));
        }
    }
}

/// The value of a single key in a [`StandardDynamicAssetCollection`]
#[derive(Debug, Clone, PartialEq)]
pub enum StandardDynamicAssetEntry {
    /// A single dynamic asset
    Single(StandardDynamicAsset),
    /// An array of dynamic assets
    Array(Vec<StandardDynamicAsset>),
    /// A map of dynamic assets
    Map(HashMap<String, StandardDynamicAsset>),
}

impl From<StandardDynamicAsset> for StandardDynamicAssetEntry {
    fn from(asset: StandardDynamicAsset) -> Self {
        StandardDynamicAssetEntry::Single(asset)
    }
}

impl From<Vec<StandardDynamicAsset>> for StandardDynamicAssetEntry {
    fn from(assets: Vec<StandardDynamicAsset>) -> Self {
        StandardDynamicAssetEntry::Array(assets)
    }
}

impl From<HashMap<String, StandardDynamicAsset>> for StandardDynamicAssetEntry {
    fn from(assets: HashMap<String, StandardDynamicAsset>) -> Self {
        StandardDynamicAssetEntry::Map(assets)
    }
}

impl StandardDynamicAssetEntry {
    /// Parse an entry from its RON source.
    fn from_raw(raw: &RawValue) -> SpannedResult<Self> {
        match raw.trim().get_ron().chars().next() {
            Some('[') => raw.into_rust().map(StandardDynamicAssetEntry::Array),
            Some('{') => parse_map(raw).map(StandardDynamicAssetEntry::Map),
            _ => raw.into_rust().map(StandardDynamicAssetEntry::Single),
        }
    }
}

impl Serialize for StandardDynamicAssetEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            StandardDynamicAssetEntry::Single(asset) => asset.serialize(serializer),
            StandardDynamicAssetEntry::Array(assets) => assets.serialize(serializer),
            StandardDynamicAssetEntry::Map(assets) => assets.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for StandardDynamicAssetEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        StandardDynamicAssetEntry::from_raw(&raw).map_err(serde::de::Error::custom)
    }
}

impl DynamicAsset for StandardDynamicAssetEntry {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            StandardDynamicAssetEntry::Single(asset) => asset.load(asset_server),
            StandardDynamicAssetEntry::Array(assets) => assets.load(asset_server),
            StandardDynamicAssetEntry::Map(assets) => assets
                .values()
                .flat_map(|asset| asset.load(asset_server))
                .collect(),
        }
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            StandardDynamicAssetEntry::Single(asset) => asset.build(world),
            StandardDynamicAssetEntry::Array(assets) => assets.build(world),
            StandardDynamicAssetEntry::Map(assets) => build_map(assets, |asset| asset.build(world)),
        }
    }

    fn load_as(&self, world: &mut World, asset_type: TypeId) -> Vec<UntypedHandle> {
        match self {
            StandardDynamicAssetEntry::Single(asset) => asset.load_as(world, asset_type),
            StandardDynamicAssetEntry::Array(assets) => assets.load_as(world, asset_type),
            StandardDynamicAssetEntry::Map(assets) => assets
                .values()
                .flat_map(|asset| asset.load_as(world, asset_type))
                .collect(),
        }
    }

    fn build_as(
        &self,
        world: &mut World,
        asset_type: TypeId,
    ) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            StandardDynamicAssetEntry::Single(asset) => asset.build_as(world, asset_type),
            StandardDynamicAssetEntry::Array(assets) => assets.build_as(world, asset_type),
            StandardDynamicAssetEntry::Map(assets) => {
                build_map(assets, |asset| asset.build_as(world, asset_type))
            }
        }
    }
}

fn build_map(
    assets: &HashMap<String, StandardDynamicAsset>,
    mut build: impl FnMut(&StandardDynamicAsset) -> Result<DynamicAssetType, anyhow::Error>,
) -> Result<DynamicAssetType, anyhow::Error> {
    let mut handles = HashMap::with_capacity(assets.len());
    for (key, asset) in assets {
        match build(asset)? {
            DynamicAssetType::Single(handle) => {
                handles.insert(key.clone(), handle);
            }
            result => anyhow::bail!(
                "The dynamic asset {asset:?} with the key '{key}' in a map has to resolve to a single handle, but resolves to {result:?}"
            ),
        }
    }
    Ok(DynamicAssetType::Map(handles))
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
                DynamicAssetType::Map(handles) => all_handles.extend(handles.into_values()),
            }
        }

        Ok(DynamicAssetType::Collection(all_handles))
    }

    fn load_as(&self, world: &mut World, asset_type: TypeId) -> Vec<UntypedHandle> {
        self.iter()
            .flat_map(|asset| asset.load_as(world, asset_type))
            .collect()
    }

    fn build_as(
        &self,
        world: &mut World,
        asset_type: TypeId,
    ) -> Result<DynamicAssetType, anyhow::Error> {
        let mut all_handles = vec![];

        for asset in self {
            match asset.build_as(world, asset_type)? {
                DynamicAssetType::Single(handle) => all_handles.push(handle),
                DynamicAssetType::Collection(handles) => all_handles.extend(handles),
                DynamicAssetType::Map(handles) => all_handles.extend(handles.into_values()),
            }
        }

        Ok(DynamicAssetType::Collection(all_handles))
    }
}

/// Registers the asset type and loader for [`StandardDynamicAssetCollection`]
pub(crate) struct StandardDynamicAssetCollectionPlugin {
    pub(crate) extensions: Vec<&'static str>,
}

impl Plugin for StandardDynamicAssetCollectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StandardDynamicAssetCollection>()
            .register_asset_loader(StandardDynamicAssetCollectionLoader {
                extensions: self.extensions.clone(),
            });
    }
}

/// Loads [`StandardDynamicAssetCollection`]s from RON files
#[derive(TypePath)]
struct StandardDynamicAssetCollectionLoader {
    extensions: Vec<&'static str>,
}

impl AssetLoader for StandardDynamicAssetCollectionLoader {
    type Asset = StandardDynamicAssetCollection;
    type Settings = ();
    type Error = StandardDynamicAssetCollectionError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let source = std::str::from_utf8(&bytes)?;
        parse_collection(source)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}

struct Pairs<'a>(Vec<(String, &'a RawValue)>);
impl<'de: 'a, 'a> Deserialize<'de> for Pairs<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PairsVisitor<'a>(std::marker::PhantomData<&'a ()>);
        impl<'de: 'a, 'a> serde::de::Visitor<'de> for PairsVisitor<'a> {
            type Value = Pairs<'a>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map from asset keys to dynamic assets")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut pairs = Vec::with_capacity(map.size_hint().unwrap_or_default());
                while let Some(pair) = map.next_entry()? {
                    pairs.push(pair);
                }
                Ok(Pairs(pairs))
            }
        }
        deserializer.deserialize_map(PairsVisitor(std::marker::PhantomData))
    }
}

/// Parse a [`StandardDynamicAssetCollection`]
fn parse_collection(
    source: &str,
) -> Result<StandardDynamicAssetCollection, StandardDynamicAssetCollectionError> {
    #[derive(Deserialize)]
    struct Collection<'a>(#[serde(borrow)] Pairs<'a>);

    let Collection(Pairs(pairs)) = ron::from_str(source)?;
    let mut collection = HashMap::with_capacity(pairs.len());
    for (key, raw) in pairs {
        let entry = StandardDynamicAssetEntry::from_raw(raw).map_err(|error| {
            StandardDynamicAssetCollectionError::Entry {
                error: Box::new(offset_error(error, source, raw)),
                key: key.clone(),
            }
        })?;
        collection.insert(key, entry);
    }
    Ok(StandardDynamicAssetCollection(collection))
}

/// Parse a map of dynamic assets with errors positioned relative to `raw`.
fn parse_map(raw: &RawValue) -> SpannedResult<HashMap<String, StandardDynamicAsset>> {
    let Pairs(pairs) = raw.into_rust()?;
    let mut map = HashMap::with_capacity(pairs.len());
    for (key, value) in pairs {
        let asset: StandardDynamicAsset = value
            .into_rust()
            .map_err(|error| offset_error(error, raw.get_ron(), value))?;
        if matches!(
            asset,
            StandardDynamicAsset::Folder { .. } | StandardDynamicAsset::Files { .. }
        ) {
            let value = value.trim();
            let error = SpannedError {
                code: ron::Error::Message(format!(
                    "The value of \"{key}\" resolves to multiple assets, but values in maps have to be single assets. Use an array instead of a map for `Folder` and `Files`"
                )),
                span: Span {
                    start: Position { line: 1, col: 1 },
                    end: end_position(value.get_ron()),
                },
            };
            return Err(offset_error(error, raw.get_ron(), value));
        }
        map.insert(key, asset);
    }
    Ok(map)
}

/// The position after the last character of `source`
fn end_position(source: &str) -> Position {
    let line = 1 + source.matches('\n').count();
    let col = 1 + source
        .rsplit('\n')
        .next()
        .map_or(0, |line| line.chars().count());
    Position { line, col }
}

fn offset_error(mut error: SpannedError, source: &str, raw: &RawValue) -> SpannedError {
    let offset = raw.get_ron().as_ptr() as usize - source.as_ptr() as usize;
    let before = &source[..offset];
    let line = before.matches('\n').count();
    let col = before
        .rsplit('\n')
        .next()
        .map_or(0, |line| line.chars().count());
    let shift = |position: &mut Position| {
        if position.line == 1 {
            position.col += col;
        }
        position.line += line;
    };
    shift(&mut error.span.start);
    shift(&mut error.span.end);
    error
}

/// Errors loading a [`StandardDynamicAssetCollection`]
#[derive(Debug)]
#[non_exhaustive]
pub enum StandardDynamicAssetCollectionError {
    /// The file could not be read
    Io(std::io::Error),
    /// The file is not valid UTF-8
    Utf8(std::str::Utf8Error),
    /// The file is not a valid RON map
    Ron(Box<SpannedError>),
    /// An entry of the collection is not a valid dynamic asset or array of dynamic assets
    Entry {
        /// The key of the invalid entry
        key: String,
        /// The error with its position in the file
        error: Box<SpannedError>,
    },
}

impl std::fmt::Display for StandardDynamicAssetCollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Could not read the file: {error}"),
            Self::Utf8(error) => write!(f, "The file is not valid UTF-8: {error}"),
            Self::Ron(error) => write!(f, "Could not parse RON: {error}"),
            Self::Entry { key, error } => write!(f, "Invalid dynamic asset \"{key}\" at {error}"),
        }
    }
}

impl std::error::Error for StandardDynamicAssetCollectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Utf8(error) => Some(error),
            Self::Ron(error) | Self::Entry { error, .. } => Some(error.as_ref()),
        }
    }
}

impl From<std::io::Error> for StandardDynamicAssetCollectionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<std::str::Utf8Error> for StandardDynamicAssetCollectionError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl From<SpannedError> for StandardDynamicAssetCollectionError {
    fn from(error: SpannedError) -> Self {
        Self::Ron(Box::new(error))
    }
}

#[cfg(test)]
#[cfg(feature = "2d")]
mod tests {
    use crate::prelude::{
        StandardDynamicAsset, StandardDynamicAssetCollection, StandardDynamicAssetEntry,
    };
    use crate::standard_dynamic_asset::parse_collection;

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
    "single": File(
        path: "audio/background.ogg",
    ),
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

        let before: StandardDynamicAssetCollection = ron::from_str(dynamic_asset_file).unwrap();
        assert_eq!(before, parse_collection(dynamic_asset_file).unwrap());
        assert!(matches!(
            before.0["single"],
            StandardDynamicAssetEntry::Single(StandardDynamicAsset::File { .. })
        ));
        assert!(matches!(
            &before.0["layouts"],
            StandardDynamicAssetEntry::Array(layouts) if layouts.len() == 4
        ));

        let serialized_dynamic_asset_file =
            ron::ser::to_string_pretty(&before, ron::ser::PrettyConfig::default().new_line("\n"))
                .unwrap();

        let after: StandardDynamicAssetCollection =
            ron::from_str(&serialized_dynamic_asset_file).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn empty_array() {
        let collection = parse_collection(r#"({"empty": []})"#).unwrap();
        assert_eq!(
            collection.0["empty"],
            StandardDynamicAssetEntry::Array(vec![])
        );
    }

    #[test]
    fn comment_before_array() {
        let collection = parse_collection(
            r#"({"files": /* list */ // of files
            [File(path: "a.ogg")]})"#,
        )
        .unwrap();
        assert!(matches!(
            collection.0["files"],
            StandardDynamicAssetEntry::Array(_)
        ));
    }

    fn error_message(dynamic_asset_file: &str) -> String {
        parse_collection(dynamic_asset_file)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn error_in_single_asset_on_first_line_of_entry() {
        assert_eq!(
            error_message(
                r#"({
    "audio": File(path: "audio/background.ogg"),
    "tree": File(pth: "images/tree.png"),
})"#
            ),
            "Invalid dynamic asset \"tree\" at 3:18-3:21: Unexpected field named `pth` in `File`, expected `path` instead"
        );
    }

    #[test]
    fn error_in_single_asset_on_later_line_of_entry() {
        assert_eq!(
            error_message(
                r#"({
    "layout": TextureAtlasLayout(
        tile_size_x: -96,
        tile_size_y: 99,
        columns: 8,
        rows: 1,
    ),
})"#
            ),
            "Invalid dynamic asset \"layout\" at 3:23-3:25: Integer is out of bounds"
        );
    }

    #[test]
    fn error_in_array() {
        assert_eq!(
            error_message(
                r#"({
    "images": [
        Image(path: "images/tree.png"),
        Imagee(path: "images/player.png"),
    ],
})"#
            ),
            "Invalid dynamic asset \"images\" at 4:9-4:15: Unexpected variant named `Imagee` in enum `StandardDynamicAsset`, expected one of `File`, `Folder`, `Files`, `Image`, `StandardMaterial`, or `TextureAtlasLayout` instead"
        );
    }

    #[test]
    fn syntax_error() {
        assert_eq!(
            error_message(
                r#"({
    "audio": File(path: "audio/background.ogg"
})"#
            ),
            "Could not parse RON: 3:1: Expected comma"
        );
    }

    #[test]
    fn map() {
        let dynamic_asset_file = r#"({
    "single": File(path: "audio/background.ogg"),
    "layouts": {
        "walk": TextureAtlasLayout(tile_size_x: 32, tile_size_y: 32, columns: 8, rows: 1),
        "jump": TextureAtlasLayout(tile_size_x: 32, tile_size_y: 64, columns: 4, rows: 1),
    },
    "empty": {},
})"#;
        let collection = parse_collection(dynamic_asset_file).unwrap();
        let StandardDynamicAssetEntry::Map(layouts) = &collection.0["layouts"] else {
            panic!("Expected a map, got {:?}", collection.0["layouts"]);
        };
        assert!(matches!(
            layouts["jump"],
            StandardDynamicAsset::TextureAtlasLayout { columns: 4, .. }
        ));
        assert_eq!(layouts.len(), 2);
        assert_eq!(
            collection.0["empty"],
            StandardDynamicAssetEntry::Map(Default::default())
        );

        let serialized_dynamic_asset_file = ron::ser::to_string_pretty(
            &collection,
            ron::ser::PrettyConfig::default().new_line("\n"),
        )
        .unwrap();
        assert_eq!(
            collection,
            parse_collection(&serialized_dynamic_asset_file).unwrap()
        );
        let deserialized: StandardDynamicAssetCollection =
            ron::from_str(&serialized_dynamic_asset_file).unwrap();
        assert_eq!(collection, deserialized);
    }

    #[test]
    fn error_in_map() {
        assert_eq!(
            error_message(
                r#"({
    "icons": {
        "play": Image(path: "images/play.png"),
        "pause": Image(path: "images/pause.png", sampler: Nearst),
    },
})"#
            ),
            "Invalid dynamic asset \"icons\" at 4:59-4:65: Unexpected variant named `Nearst` in enum `ImageSamplerType`, expected either `Nearest` or `Linear` instead"
        );
    }

    #[test]
    fn folder_in_map() {
        assert_eq!(
            error_message(
                r#"({
    "icons": {
        "play": Image(path: "images/play.png"),
        "all": Folder(
            path: "images",
        ),
    },
})"#
            ),
            "Invalid dynamic asset \"icons\" at 4:16-6:10: The value of \"all\" resolves to multiple assets, but values in maps have to be single assets. Use an array instead of a map for `Folder` and `Files`"
        );
    }

    fn serialize_and_deserialize(dynamic_asset_file: &'static str) {
        let before: StandardDynamicAssetCollection = ron::from_str(dynamic_asset_file).unwrap();

        let serialized_dynamic_asset_file =
            ron::ser::to_string_pretty(&before, ron::ser::PrettyConfig::default().new_line("\n"))
                .unwrap();
        assert_eq!(dynamic_asset_file, &serialized_dynamic_asset_file);
    }
}
