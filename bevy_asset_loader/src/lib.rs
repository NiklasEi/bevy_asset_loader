//! The goal of this crate is to offer an easy way for bevy games to load all their assets in a loading State.
//!
//! `bevy_asset_loader` introduces the derivable trait [AssetCollection]. Structs with asset handles
//! can be automatically loaded during a configurable loading [State]. Afterwards they will be inserted as
//! resources containing loaded handles and the plugin will switch to a second configurable [State].
//!
//! ```edition2018
//! # use bevy_asset_loader::{AssetLoader, AssetCollection};
//! # use bevy::prelude::*;
//! # use bevy::asset::AssetPlugin;
//! fn main() {
//! let mut app = App::build();
//!     AssetLoader::new(GameState::Loading, GameState::Next)
//!         .with_collection::<AudioAssets>()
//!         .with_collection::<TextureAssets>()
//!         .build(&mut app);
//!     app
//!         .add_state(GameState::Loading)
//!         //.add_plugins(DefaultPlugins)
//!         .add_system_set(SystemSet::on_update(GameState::Next)
//!             .with_system(use_asset_handles.system())
//!         )
//!         # .add_plugins(MinimalPlugins)
//!         # .add_plugin(AssetPlugin::default())
//!         # .set_runner(|mut app| app.schedule.run(&mut app.world))
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
//! pub struct TextureAssets {
//!     #[asset(path = "textures/player.png")]
//!     pub player: Handle<Texture>,
//!     #[asset(path = "textures/tree.png")]
//!     pub tree: Handle<Texture>,
//! }
//!
//! // since this function runs in [MyState::Next], we know our assets are
//! // loaded and [MyAudioAssets] is a resource
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

pub use bevy_asset_loader_derive::AssetCollection;

use bevy::app::AppBuilder;
use bevy::asset::{AssetServer, HandleUntyped, LoadState};
use bevy::ecs::component::Component;
use bevy::ecs::schedule::State;
use bevy::ecs::system::IntoSystem;
use bevy::prelude::{Commands, Res, ResMut, SystemSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// Trait to mark a struct as a collection of assets
///
/// Derive is supported for structs with named fields.
/// Each field needs to be annotated with ``#[asset(path = "path/to/asset.file")]``
/// ```edition2018
/// # use bevy_asset_loader::AssetCollection;
/// # use bevy::prelude::*;
/// #[derive(AssetCollection)]
/// struct MyAssets {
///     #[asset(path = "player.png")]
///     player: Handle<Texture>,
///     #[asset(path = "tree.png")]
///     tree: Handle<Texture>
/// }
/// ```
pub trait AssetCollection: Component {
    /// Create a new AssetCollection from the [bevy::asset::AssetServer]
    fn create(asset_server: &Res<AssetServer>) -> Self;
    /// Start loading all the assets in the collection
    fn load(asset_server: &Res<AssetServer>) -> Vec<HandleUntyped>;
}

struct LoadingAssetHandles<A: Component> {
    handles: Vec<HandleUntyped>,
    marker: PhantomData<A>,
}

struct AssetLoaderConfiguration<T> {
    next: T,
    count: usize,
}

fn start_loading<Assets: AssetCollection>(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadingAssetHandles {
        handles: Assets::load(&asset_server),
        marker: PhantomData::<Assets>,
    })
}

fn check_loading_state<T: Component + Debug + Clone + Eq + Hash, Assets: AssetCollection>(
    mut commands: Commands,
    mut state: ResMut<State<T>>,
    mut config: ResMut<AssetLoaderConfiguration<T>>,
    asset_server: Res<AssetServer>,
    loading_asset_handles: Option<Res<LoadingAssetHandles<Assets>>>,
) {
    if let Some(loading_asset_handles) = loading_asset_handles {
        let load_state = asset_server
            .get_group_load_state(loading_asset_handles.handles.iter().map(|handle| handle.id));
        if load_state == LoadState::Loaded {
            commands.insert_resource(Assets::create(&asset_server));
            commands.remove_resource::<LoadingAssetHandles<Assets>>();
            if config.count == 1 {
                commands.remove_resource::<AssetLoaderConfiguration<T>>();
                state
                    .set(config.next.clone())
                    .expect("Failed to set next State");
            } else {
                config.count -= 1;
            }
        }
    }
}

/// A Bevy plugin to configure automatic asset loading
///
/// ```edition2018
/// # use bevy_asset_loader::{AssetLoader, AssetCollection};
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// fn main() {
/// let mut app = App::build();
///     AssetLoader::new(GameState::Loading, GameState::Menu)
///         .with_collection::<AudioAssets>()
///         .with_collection::<TextureAssets>()
///         .build(&mut app);
///
///     app.add_state(GameState::Loading)
///         .add_plugins(MinimalPlugins)
///         .add_plugin(AssetPlugin::default())
///         .add_system_set(SystemSet::on_enter(GameState::Menu)
///             .with_system(play_audio.system())
///         )
/// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
///         .run();
/// }
///
/// fn play_audio(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
///     audio.play(audio_assets.flying.clone());
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
/// pub struct TextureAssets {
///     #[asset(path = "textures/player.png")]
///     pub player: Handle<Texture>,
///     #[asset(path = "textures/tree.png")]
///     pub tree: Handle<Texture>,
/// }
/// ```
pub struct AssetLoader<T> {
    next: T,
    load: SystemSet,
    check: SystemSet,
    collection_count: usize,
}

impl<T> AssetLoader<T>
where
    T: Component + Debug + Clone + Eq + Hash,
{
    /// Create a new [AssetLoader]
    ///
    /// This function takes two States. During the first all assets will be loaded and the
    /// collections will be inserted as resources. Then the second state is set in your Bevy App.
    /// ```edition2018
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::build();
    ///     AssetLoader::new(GameState::Loading, GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<TextureAssets>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default())
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
    /// # pub struct TextureAssets {
    /// #     #[asset(path = "textures/player.png")]
    /// #     pub player: Handle<Texture>,
    /// #     #[asset(path = "textures/tree.png")]
    /// #     pub tree: Handle<Texture>,
    /// # }
    /// ```
    pub fn new(load: T, next: T) -> AssetLoader<T> {
        Self {
            next,
            load: SystemSet::on_enter(load.clone()),
            check: SystemSet::on_update(load),
            collection_count: 0,
        }
    }

    /// Add an [AssetCollection] to the [AssetLoader]
    ///
    /// The added collection will be loaded and inserted into your Bevy App as a resource.
    /// ```edition2018
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::build();
    ///     AssetLoader::new(GameState::Loading, GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<TextureAssets>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default())
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
    /// # pub struct TextureAssets {
    /// #     #[asset(path = "textures/player.png")]
    /// #     pub player: Handle<Texture>,
    /// #     #[asset(path = "textures/tree.png")]
    /// #     pub tree: Handle<Texture>,
    /// # }
    /// ```
    pub fn with_collection<A: AssetCollection>(mut self) -> Self {
        self.load = self.load.with_system(start_loading::<A>.system());
        self.check = self.check.with_system(check_loading_state::<T, A>.system());
        self.collection_count += 1;

        self
    }

    /// Finish configuring the [AssetLoader]
    ///
    /// Calling this function is required to set up the asset loading.
    /// ```edition2018
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     let mut app = App::build();
    ///     AssetLoader::new(GameState::Loading, GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<TextureAssets>()
    ///         .build(&mut app);
    /// #   app
    /// #       .add_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_plugin(AssetPlugin::default())
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
    /// # pub struct TextureAssets {
    /// #     #[asset(path = "textures/player.png")]
    /// #     pub player: Handle<Texture>,
    /// #     #[asset(path = "textures/tree.png")]
    /// #     pub tree: Handle<Texture>,
    /// # }
    /// ```
    pub fn build(self, app: &mut AppBuilder) {
        app.add_system_set(self.load)
            .add_system_set(self.check)
            .insert_resource(AssetLoaderConfiguration::<T> {
                count: self.collection_count,
                next: self.next,
            });
    }
}
