use bevy::utils::HashMap;
use std::fmt::Debug;

use bevy::asset::{AssetServer, HandleUntyped};
use bevy::ecs::world::World;

#[cfg(feature = "dynamic_assets")]
use bevy::ecs::schedule::StateData;
#[cfg(feature = "dynamic_assets")]
use bevy::reflect::TypeUuid;
#[cfg(feature = "dynamic_assets")]
use std::marker::PhantomData;

/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
#[derive(Debug)]
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

/// Different typed that can generate the asset field value of a dynamic asset
pub enum DynamicAssetType {
    /// Dynamic asset that is defined by a single handle
    Single(HandleUntyped),
    /// Dynamic asset that is defined by multiple handles
    Collection(Vec<HandleUntyped>),
}

/// Any type implementing this trait can be assigned to asset keys as part of a dynamic
/// asset collection.
pub trait DynamicAsset: Debug + Send + Sync {
    /// Return handles to all required asset paths
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped>;

    /// Return the handle(s) defining this asset
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error>;
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

/// Resource to dynamically resolve keys to asset paths.
///
/// This resource is set by a [`LoadingState`](crate::loading_state::LoadingState) and is read when entering the corresponding Bevy [`State`](::bevy::ecs::schedule::State).
/// If you want to manage your dynamic assets manually, they should be configured in a previous [`State`](::bevy::ecs::schedule::State).
///
/// ```edition2021
/// # use bevy::prelude::*;
/// # use bevy_asset_loader::prelude::*;
/// fn choose_character(
///     mut state: ResMut<State<GameState>>,
///     mut asset_keys: ResMut<DynamicAssets>,
///     mouse_input: Res<Input<MouseButton>>,
/// ) {
///     if mouse_input.just_pressed(MouseButton::Left) {
///         asset_keys.register_asset(
///             "character",
///             Box::new(StandardDynamicAsset::File {
///                 path: "images/female_adventurer.png".to_owned(),
///             }),
///         );
///     } else if mouse_input.just_pressed(MouseButton::Right) {
///         asset_keys.register_asset(
///             "character",
///             Box::new(StandardDynamicAsset::File {
///                 path: "images/zombie.png".to_owned(),
///             }),
///         );
///     } else {
///         return;
///     }
///
///     state
///         .set(GameState::Loading)
///         .expect("Failed to change state");
/// }
///
/// #[derive(AssetCollection)]
/// struct ImageAssets {
///     #[asset(key = "character")]
///     player: Handle<Image>,
/// }
/// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
/// # enum GameState {
/// #     Loading,
/// #     Menu
/// # }
/// ```
#[derive(Default)]
pub struct DynamicAssets {
    key_asset_map: HashMap<String, Box<dyn DynamicAsset>>,
}

impl DynamicAssets {
    /// Get the asset corresponding to the given key.
    pub fn get_asset(&self, key: &str) -> Option<&dyn DynamicAsset> {
        self.key_asset_map.get(key).map(|boxed| boxed.as_ref())
    }

    /// Set the corresponding dynamic asset for the given key.
    ///
    /// In case the key is already known, its value will be overwritten.
    /// ```edition2021
    /// # use bevy::prelude::*;
    /// # use bevy_asset_loader::prelude::*;
    /// fn choose_character(
    ///     mut state: ResMut<State<GameState>>,
    ///     mut asset_keys: ResMut<DynamicAssets>,
    ///     mouse_input: Res<Input<MouseButton>>,
    /// ) {
    ///     if mouse_input.just_pressed(MouseButton::Left) {
    ///         asset_keys.register_asset("character", Box::new(StandardDynamicAsset::File{path: "images/female_adventurer.png".to_owned()}))
    ///     } else if mouse_input.just_pressed(MouseButton::Right) {
    ///         asset_keys.register_asset("character", Box::new(StandardDynamicAsset::File{path: "images/zombie.png".to_owned()}))
    ///     } else {
    ///         return;
    ///     }
    ///
    ///     state
    ///         .set(GameState::Loading)
    ///         .expect("Failed to change state");
    /// }
    ///
    /// #[derive(AssetCollection)]
    /// struct ImageAssets {
    ///     #[asset(key = "character")]
    ///     player: Handle<Image>,
    /// }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
    /// #     Loading,
    /// #     Menu
    /// # }
    /// ```
    pub fn register_asset<K: Into<String>>(&mut self, key: K, asset: Box<dyn DynamicAsset>) {
        self.key_asset_map.insert(key.into(), asset);
    }
}

/// ToDo
pub trait DynamicAssetCollection {
    /// Register all dynamic assets inside the collection in the [`DynamicAssets`] resource.
    fn register(self, dynamic_assets: &mut DynamicAssets);
}

/// Resource keeping track of dynamic asset collection files for different loading states
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic_assets")))]
#[cfg(feature = "dynamic_assets")]
pub struct DynamicAssetCollections<State: StateData> {
    /// Dynamic asset collection files for different loading states.
    ///
    /// The file lists get loaded and emptied at the beginning of the loading states.
    /// Make sure to add any file you would like to load before entering the loading state!
    pub files: HashMap<State, Vec<String>>,
    pub(crate) _marker: PhantomData<State>,
}

#[cfg(feature = "dynamic_assets")]
impl<State: StateData> Default for DynamicAssetCollections<State> {
    fn default() -> Self {
        DynamicAssetCollections {
            files: HashMap::default(),
            _marker: PhantomData::default(),
        }
    }
}

/// The asset defining a mapping from asset keys to dynamic assets
///
/// These assets are loaded at the beginning of a loading state
/// and combined in [`DynamicAssets`](DynamicAssets).
#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "2df82c01-9c71-4aa8-adc4-71c5824768f1"]
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic_assets")))]
#[cfg(feature = "dynamic_assets")]
pub struct StandardDynamicAssetCollection(pub HashMap<String, StandardDynamicAsset>);

#[cfg(feature = "dynamic_assets")]
impl DynamicAssetCollection for StandardDynamicAssetCollection {
    fn register(self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0 {
            dynamic_assets.register_asset(key, Box::new(asset));
        }
    }
}
