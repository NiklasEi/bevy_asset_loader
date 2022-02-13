//! The goal of this crate is to offer an easy way for bevy games to load all their assets in a loading [`State`](bevy_ecs::schedule::State).
//!
//! `bevy_asset_loader` introduces the derivable trait [`AssetCollection`]. Structs with asset handles
//! can be automatically loaded during a configurable loading [`State`](bevy_ecs::schedule::State). Afterwards they will be inserted as
//! resources containing loaded handles and the plugin will switch to a second configurable [`State`](bevy_ecs::schedule::State).
//!
//! ```edition2021
//! # use bevy_asset_loader::{AssetLoader, AssetCollection};
//! # use bevy::prelude::*;
//! # use bevy::asset::AssetPlugin;
//! fn main() {
//!     let mut app = App::new();
//!     app
//! # /*
//!         .add_plugins(DefaultPlugins)
//! # */
//! #       .add_plugins(MinimalPlugins)
//! #       .add_plugin(AssetPlugin::default());
//!     AssetLoader::new(GameState::Loading)
//!         .continue_to_state(GameState::Next)
//!         .with_collection::<AudioAssets>()
//!         .with_collection::<ImageAssets>()
//!         .build(&mut app);
//!     app
//!         .add_state(GameState::Loading)
//!         .add_system_set(SystemSet::on_update(GameState::Next)
//!             .with_system(use_asset_handles.system())
//!         )
//! #       .set_runner(|mut app| app.schedule.run(&mut app.world))
//!         .run();
//! }
//!
//! #[derive(AssetCollection)]
//! struct AudioAssets {
//!     #[asset(path = "audio/background.ogg")]
//!     background: Handle<AudioSource>,
//!     #[asset(path = "audio/plop.ogg")]
//!     plop: Handle<AudioSource>
//! }
//!
//! #[derive(AssetCollection)]
//! pub struct ImageAssets {
//!     #[asset(path = "images/player.png")]
//!     pub player: Handle<Image>,
//!     #[asset(path = "images/tree.png")]
//!     pub tree: Handle<Image>,
//! }
//!
//! // since this function runs in MyState::Next, we know our assets are
//! // loaded and their handles are in the resource AudioAssets
//! fn use_asset_handles(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
//!     audio.play(audio_assets.background.clone());
//! }
//!
//! #[derive(Clone, Eq, PartialEq, Debug, Hash)]
//! enum GameState {
//!     Loading,
//!     Next
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use std::marker::PhantomData;

use bevy::app::App;
#[cfg(feature = "dynamic_assets")]
use bevy::asset::Handle;
use bevy::asset::HandleUntyped;
use bevy::ecs::prelude::IntoExclusiveSystem;
use bevy::ecs::schedule::ExclusiveSystemDescriptorCoercion;
use bevy::ecs::schedule::StateData;
use bevy::prelude::{FromWorld, SystemSet, World};
use bevy::utils::HashMap;
#[cfg(feature = "dynamic_assets")]
use bevy_asset_ron::RonAssetPlugin;

pub use bevy_asset_loader_derive::AssetCollection;
#[cfg(feature = "dynamic_assets")]
use dynamic_asset::DynamicAssetCollection;

pub use crate::dynamic_asset::DynamicAsset;

mod dynamic_asset;
mod systems;

/// Trait to mark a struct as a collection of assets
///
/// Derive is supported for structs with named fields.
/// ```edition2021
/// # use bevy_asset_loader::AssetCollection;
/// # use bevy::prelude::*;
/// #[derive(AssetCollection)]
/// struct MyAssets {
///     #[asset(path = "player.png")]
///     player: Handle<Image>,
///     #[asset(path = "tree.png")]
///     tree: Handle<Image>
/// }
/// ```
pub trait AssetCollection: Send + Sync + 'static {
    /// Create a new asset collection from the [`AssetServer`](bevy_asset::AssetServer)
    fn create(world: &mut World) -> Self;
    /// Start loading all the assets in the collection
    fn load(world: &mut World) -> Vec<HandleUntyped>;
}

/// Extension trait for [`App`](bevy::app::App) enabling initialisation of [asset collections](AssetCollection)
pub trait AssetCollectionApp {
    /// Initialise an [`AssetCollection`]
    ///
    /// This function does not give any guaranties about the loading status of the asset handles.
    /// If you want to use a loading state, you do not need this function! Instead use an [`AssetLoader`]
    /// and add collections to it to be prepared during the loading state.
    fn init_collection<A: AssetCollection>(&mut self) -> &mut Self;
}

impl AssetCollectionApp for App {
    fn init_collection<Collection>(&mut self) -> &mut Self
    where
        Collection: AssetCollection,
    {
        if !self.world.contains_resource::<Collection>() {
            // This resource is required for loading a collection
            // Since bevy_asset_loader does not have a "real" Plugin,
            // we need to make sure the resource exists here
            self.init_resource::<AssetKeys>();
            // make sure the assets start to load
            let _ = Collection::load(&mut self.world);
            let resource = Collection::create(&mut self.world);
            self.insert_resource(resource);
        }
        self
    }
}

/// Extension trait for [`World`](bevy::ecs::world::World) enabling initialisation of [asset collections](AssetCollection)
pub trait AssetCollectionWorld {
    /// Initialise an [`AssetCollection`]
    ///
    /// This function does not give any guaranties about the loading status of the asset handles.
    /// If you want to use a loading state, you do not need this function! Instead use an [`AssetLoader`]
    /// and add collections to it to be prepared during the loading state.
    fn init_collection<A: AssetCollection>(&mut self);
}

impl AssetCollectionWorld for World {
    fn init_collection<A: AssetCollection>(&mut self) {
        if self.get_resource::<A>().is_none() {
            if self.get_resource::<AssetKeys>().is_none() {
                // This resource is required for loading a collection
                // Since bevy_asset_loader does not have a "real" Plugin,
                // we need to make sure the resource exists here
                self.insert_resource(AssetKeys::default());
            }
            // make sure the assets start to load
            let _ = A::load(self);
            let collection = A::create(self);
            self.insert_resource(collection);
        }
    }
}

struct LoadingAssetHandles<A: AssetCollection> {
    handles: Vec<HandleUntyped>,
    marker: PhantomData<A>,
}

struct AssetLoaderConfiguration<State> {
    configuration: HashMap<State, LoadingConfiguration<State>>,
    phase: HashMap<State, LoadingStatePhase>,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_handles: Vec<Handle<DynamicAssetCollection>>,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_files: HashMap<State, Vec<String>>,
}

impl<State> Default for AssetLoaderConfiguration<State> {
    fn default() -> Self {
        AssetLoaderConfiguration {
            configuration: HashMap::default(),
            phase: HashMap::default(),
            #[cfg(feature = "dynamic_assets")]
            asset_collection_handles: vec![],
            #[cfg(feature = "dynamic_assets")]
            asset_collection_files: HashMap::default(),
        }
    }
}

impl<State: StateData> AssetLoaderConfiguration<State> {
    /// Get all asset collection files registered for the given state
    ///
    /// The files can be loaded as [`DynamicAssetCollection`](crate::dynamic_asset::DynamicAssetCollection) assets.
    #[cfg(feature = "dynamic_assets")]
    pub fn get_asset_collection_files(&mut self, state: &State) -> Vec<String> {
        self.asset_collection_files
            .remove(state)
            .unwrap()
            .iter()
            .map(|file| file.to_owned())
            .collect()
    }
}

#[derive(Clone, Debug)]
enum LoadingStatePhase {
    #[cfg(feature = "dynamic_assets")]
    PreparingAssetKeys,
    StartLoading,
    Loading,
}

struct LoadingConfiguration<T> {
    next: Option<T>,
    count: usize,
}

/// Resource to dynamically resolve keys to asset paths.
///
/// This resource is set by the [`AssetLoader`] and is read when entering a loading state.
/// You should set your desired asset key and paths in a previous [`State`](bevy_ecs::schedule::State).
///
/// ```edition2021
/// # use bevy::prelude::*;
/// # use bevy_asset_loader::{AssetKeys, AssetCollection, DynamicAsset};
/// fn choose_character(
///     mut state: ResMut<State<GameState>>,
///     mut asset_keys: ResMut<AssetKeys>,
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
pub struct AssetKeys {
    key_asset_map: HashMap<String, DynamicAsset>,
}

impl AssetKeys {
    /// Get the asset corresponding to the given key.
    pub fn get_asset(&self, key: &str) -> Option<&DynamicAsset> {
        self.key_asset_map.get(key)
    }

    /// Set the corresponding dynamic asset for the given key.
    ///
    /// In case the key is already known, its value will be overwritten.
    /// ```edition2021
    /// # use bevy::prelude::*;
    /// # use bevy_asset_loader::{AssetKeys, AssetCollection, DynamicAsset};
    /// fn choose_character(
    ///     mut state: ResMut<State<GameState>>,
    ///     mut asset_keys: ResMut<AssetKeys>,
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
}

/// A Bevy plugin to configure automatic asset loading
///
/// ```edition2021
/// # use bevy_asset_loader::{AssetLoader, AssetCollection};
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// fn main() {
///     let mut app = App::new();
///     app
///         .add_plugins(MinimalPlugins)
///         .add_plugin(AssetPlugin::default());
///     AssetLoader::new(GameState::Loading)
///         .continue_to_state(GameState::Menu)
///         .with_collection::<AudioAssets>()
///         .with_collection::<ImageAssets>()
///         .build(&mut app);
///
///     app.add_state(GameState::Loading)
///         .add_system_set(SystemSet::on_enter(GameState::Menu)
///             .with_system(play_audio.system())
///         )
/// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
///         .run();
/// }
///
/// fn play_audio(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
///     audio.play(audio_assets.background.clone());
/// }
///
/// #[derive(Clone, Eq, PartialEq, Debug, Hash)]
/// enum GameState {
///     Loading,
///     Menu
/// }
///
/// #[derive(AssetCollection)]
/// pub struct AudioAssets {
///     #[asset(path = "audio/background.ogg")]
///     pub background: Handle<AudioSource>,
/// }
///
/// #[derive(AssetCollection)]
/// pub struct ImageAssets {
///     #[asset(path = "images/player.png")]
///     pub player: Handle<Image>,
///     #[asset(path = "images/tree.png")]
///     pub tree: Handle<Image>,
/// }
/// ```
pub struct AssetLoader<State> {
    next_state: Option<State>,
    loading_state: State,
    keys: HashMap<String, DynamicAsset>,
    on_enter: SystemSet,
    on_update: SystemSet,
    on_exit: SystemSet,
    collection_count: usize,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_file_ending: &'static str,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_files: Vec<String>,
}

impl<State> AssetLoader<State>
where
    State: StateData,
{
    /// Create a new [`AssetLoader`]
    ///
    /// This function takes a [`State`](bevy_ecs::schedule::State) during which all asset collections will
    /// be loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<ImageAssets>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct AudioAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct ImageAssets {
    /// #     #[asset(path = "images/player.png")]
    /// #     pub player: Handle<Image>,
    /// #     #[asset(path = "images/tree.png")]
    /// #     pub tree: Handle<Image>,
    /// # }
    /// ```
    pub fn new(load: State) -> AssetLoader<State> {
        Self {
            next_state: None,
            loading_state: load.clone(),
            keys: HashMap::default(),
            on_enter: SystemSet::on_enter(load.clone()),
            on_update: SystemSet::on_update(load.clone()),
            on_exit: SystemSet::on_exit(load),
            collection_count: 0,
            #[cfg(feature = "dynamic_assets")]
            asset_collection_file_ending: "assets",
            #[cfg(feature = "dynamic_assets")]
            asset_collection_files: vec![],
        }
    }

    /// The [`AssetLoader`] will set this [`State`](bevy_ecs::schedule::State) after all asset collections
    /// are loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<ImageAssets>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct AudioAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct ImageAssets {
    /// #     #[asset(path = "images/player.png")]
    /// #     pub player: Handle<Image>,
    /// #     #[asset(path = "images/tree.png")]
    /// #     pub tree: Handle<Image>,
    /// # }
    /// ```
    pub fn continue_to_state(mut self, next: State) -> Self {
        self.next_state = Some(next);

        self
    }

    /// Register an asset collection file to be loaded and used to define dynamic assets.
    ///
    /// The file will be loaded as [`DynamicAssetCollection`](crate::dynamic_asset::DynamicAssetCollection).
    /// It's mapping of asset keys to asset configurations can be used for dynamic assets.
    ///
    /// See the `dynamic_asset_ron` example.
    #[cfg(feature = "dynamic_assets")]
    pub fn with_asset_collection_file(mut self, asset_collection_file_path: &str) -> Self {
        self.asset_collection_files
            .push(asset_collection_file_path.to_owned());

        self
    }

    /// Add an [`AssetCollection`] to the [`AssetLoader`]
    ///
    /// The added collection will be loaded and inserted into your Bevy app as a resource.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<ImageAssets>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct AudioAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct ImageAssets {
    /// #     #[asset(path = "images/player.png")]
    /// #     pub player: Handle<Image>,
    /// #     #[asset(path = "images/tree.png")]
    /// #     pub tree: Handle<Image>,
    /// # }
    /// ```
    pub fn with_collection<A: AssetCollection>(mut self) -> Self {
        self.on_update = self
            .on_update
            .with_system(systems::loading_state::<State, A>.exclusive_system());
        self.collection_count += 1;

        self
    }

    /// Insert a map of asset keys with corresponding assets
    pub fn add_keys(mut self, mut keys: HashMap<String, DynamicAsset>) -> Self {
        keys.drain().for_each(|(key, value)| {
            self.keys.insert(key, value);
        });

        self
    }

    /// Add any [`FromWorld`](bevy_ecs::world::FromWorld) resource to be initialized after all asset collections are loaded.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<TextureForAtlas>()
    ///         .init_resource::<TextureAtlasFromWorld>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # struct TextureAtlasFromWorld {
    /// #     atlas: Handle<TextureAtlas>
    /// # }
    /// # impl FromWorld for TextureAtlasFromWorld {
    /// #     fn from_world(world: &mut World) -> Self {
    /// #         let cell = world.cell();
    /// #         let assets = cell.get_resource::<TextureForAtlas>().expect("TextureForAtlas not loaded");
    /// #         let mut atlases = cell.get_resource_mut::<Assets<TextureAtlas>>().expect("TextureAtlases missing");
    /// #         TextureAtlasFromWorld {
    /// #             atlas: atlases.add(TextureAtlas::from_grid(assets.array.clone(), Vec2::new(250., 250.), 1, 4))
    /// #         }
    /// #     }
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct TextureForAtlas {
    /// #     #[asset(path = "images/female_adventurer.ogg")]
    /// #     pub array: Handle<Image>,
    /// # }
    /// ```
    pub fn init_resource<A: FromWorld + Send + Sync + 'static>(mut self) -> Self {
        self.on_exit = self
            .on_exit
            .with_system(systems::init_resource::<A>.exclusive_system());

        self
    }

    /// Finish configuring the [`AssetLoader`]
    ///
    /// Calling this function is required to set up the asset loading.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<ImageAssets>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct AudioAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// # #[derive(AssetCollection)]
    /// # pub struct ImageAssets {
    /// #     #[asset(path = "images/player.png")]
    /// #     pub player: Handle<Image>,
    /// #     #[asset(path = "images/tree.png")]
    /// #     pub tree: Handle<Image>,
    /// # }
    /// ```
    #[allow(unused_mut)]
    pub fn build(mut self, app: &mut App) {
        if !app
            .world
            .contains_resource::<AssetLoaderConfiguration<State>>()
        {
            app.world
                .insert_resource(AssetLoaderConfiguration::<State>::default());
        }
        let config = LoadingConfiguration {
            next: self.next_state.clone(),
            count: 0,
        };
        {
            let mut asset_loader_configuration = app
                .world
                .get_resource_mut::<AssetLoaderConfiguration<State>>()
                .unwrap();
            asset_loader_configuration
                .configuration
                .insert(self.loading_state.clone(), config);
            asset_loader_configuration
                .phase
                .insert(self.loading_state.clone(), LoadingStatePhase::StartLoading);
            #[cfg(feature = "dynamic_assets")]
            asset_loader_configuration
                .asset_collection_files
                .insert(self.loading_state.clone(), self.asset_collection_files);
        }
        #[cfg(feature = "dynamic_assets")]
        {
            app.add_plugin(RonAssetPlugin::<DynamicAssetCollection>::new(&[
                self.asset_collection_file_ending
            ]));
            self.on_enter = self.on_enter.with_system(
                dynamic_asset::prepare_asset_keys::<State>
                    .exclusive_system()
                    .at_start(),
            );
        }
        self.on_update = self
            .on_update
            .with_system(systems::phase::<State>.exclusive_system().at_end());
        app.insert_resource(AssetKeys { key_asset_map: self.keys });
        app.add_system_set(self.on_enter)
            .add_system_set(self.on_update)
            .add_system_set(self.on_exit);
    }
}

#[cfg(feature = "render")]
#[doc = include_str!("../../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
