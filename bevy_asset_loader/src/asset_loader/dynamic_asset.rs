use bevy::utils::HashMap;

#[cfg(feature = "dynamic_assets")]
use bevy::reflect::TypeUuid;

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

impl DynamicAsset {
    /// Path to the asset file of the dynamic asset
    pub fn get_file_path(&self) -> &str {
        match self {
            DynamicAsset::File { path } => path,
            #[cfg(feature = "3d")]
            DynamicAsset::StandardMaterial { path } => path,
            #[cfg(feature = "2d")]
            DynamicAsset::TextureAtlas { path, .. } => path,
        }
    }
}

/// Resource to dynamically resolve keys to asset paths.
///
/// This resource is set by the [`AssetLoader`] and is read when entering a loading state.
/// You should set your desired asset key and paths in a previous [`State`](bevy_ecs::schedule::State).
///
/// ```edition2021
/// # use bevy::prelude::*;
/// # use bevy_asset_loader::{DynamicAssets, AssetCollection, DynamicAsset};
/// fn choose_character(
///     mut state: ResMut<State<GameState>>,
///     mut asset_keys: ResMut<DynamicAssets>,
///     mouse_input: Res<Input<MouseButton>>,
/// ) {
///     if mouse_input.just_pressed(MouseButton::Left) {
///         asset_keys.register_asset(
///             "character",
///             DynamicAsset::File {
///                 path: "images/female_adventurer.png".to_owned(),
///             },
///         );
///     } else if mouse_input.just_pressed(MouseButton::Right) {
///         asset_keys.register_asset(
///             "character",
///             DynamicAsset::File {
///                 path: "images/zombie.png".to_owned(),
///             },
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
    pub(crate) key_asset_map: HashMap<String, DynamicAsset>,
}

impl DynamicAssets {
    /// Get the asset corresponding to the given key.
    pub fn get_asset(&self, key: &str) -> Option<&DynamicAsset> {
        self.key_asset_map.get(key)
    }

    /// Set the corresponding dynamic asset for the given key.
    ///
    /// In case the key is already known, its value will be overwritten.
    /// ```edition2021
    /// # use bevy::prelude::*;
    /// # use bevy_asset_loader::{DynamicAssets, AssetCollection, DynamicAsset};
    /// fn choose_character(
    ///     mut state: ResMut<State<GameState>>,
    ///     mut asset_keys: ResMut<DynamicAssets>,
    ///     mouse_input: Res<Input<MouseButton>>,
    /// ) {
    ///     if mouse_input.just_pressed(MouseButton::Left) {
    ///         asset_keys.register_asset("character", DynamicAsset::File{path: "images/female_adventurer.png".to_owned()})
    ///     } else if mouse_input.just_pressed(MouseButton::Right) {
    ///         asset_keys.register_asset("character", DynamicAsset::File{path: "images/zombie.png".to_owned()})
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
    pub fn register_asset<K: Into<String>>(&mut self, key: K, asset: DynamicAsset) {
        self.key_asset_map.insert(key.into(), asset);
    }

    /// Register all asset keys and their values
    #[cfg(feature = "dynamic_assets")]
    pub fn register_dynamic_collection(
        &mut self,
        dynamic_asset_collection: DynamicAssetCollection,
    ) {
        for (key, asset) in dynamic_asset_collection.0 {
            self.key_asset_map.insert(key, asset);
        }
    }
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "2df82c01-9c71-4aa8-adc4-71c5824768f1"]
#[cfg(feature = "dynamic_assets")]
pub struct DynamicAssetCollection(pub(crate) HashMap<String, DynamicAsset>);
