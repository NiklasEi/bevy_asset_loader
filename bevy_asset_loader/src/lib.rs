//! The goal of this crate is to offer an easy way for bevy games to load all their assets.
//!
//! ```edition2018
//! # use bevy_asset_loader::{AssetLoaderPlugin, AssetCollection};
//! # use bevy_kira_audio::{AudioPlugin, AudioSource, Audio};
//! # use bevy::prelude::*;
//! fn main() {
//!     App::build()
//!         .add_state(MyStates::Load)
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(AudioPlugin)
//!         .add_plugin(AssetLoaderPlugin::<MyAudioAssets, _>::new(
//!             MyStates::Load,
//!             MyStates::Next,
//!         ))
//!         .add_system_set(SystemSet::on_update(MyStates::Next).with_system(use_asset_handles.system()))
//!         .run();
//! }
//!
//! #[derive(AssetCollection)]
//! struct MyAudioAssets {
//!     #[asset(path = "walking.ogg")]
//!     walking: Handle<AudioSource>,
//!     #[asset(path = "flying.ogg")]
//!     flying: Handle<AudioSource>
//! }
//!
//! // since this function runs in [MyState::Next], we know our assets are
//! // loaded and [MyAudioAssets] is a resource
//! fn use_asset_handles(audio_assets: Res<MyAudioAssets>, audio: Res<Audio>) {
//!     audio.play(audio_assets.flying.clone());
//! }
//!
//! #[derive(Clone, Eq, PartialEq, Debug, Hash)]
//! enum MyStates {
//!     Load,
//!     Next
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

pub use bevy_asset_loader_derive::AssetCollection;

use bevy::app::{AppBuilder, Plugin};
use bevy::asset::{AssetServer, HandleId, HandleUntyped, LoadState};
use bevy::ecs::component::Component;
use bevy::ecs::schedule::State;
use bevy::ecs::system::IntoSystem;
use bevy::prelude::{Commands, Res, ResMut, SystemSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// A Bevy plugin to load asset collections
///
/// Configure it with a struct implementing [AssetCollection] and two [bevy::ecs::schedule::State]s
/// ```edition2018
/// # use bevy_asset_loader::{AssetLoaderPlugin, AssetCollection};
/// # use bevy::prelude::*;
/// fn main() {
///    let mut app = App::build();
///    app
///         .add_plugins(DefaultPlugins)
///         .add_plugin(AssetLoaderPlugin::<MyAssets, _>::new(
///             MyStates::Load,
///             MyStates::Next,
///         ));
///    app.run();
/// }
///
/// # #[derive(AssetCollection)]
/// # struct MyAssets {
/// #     #[asset(path = "player.png")]
/// #     player: Handle<Texture>,
/// #     #[asset(path = "tree.png")]
/// #     tree: Handle<Texture>
/// # }
/// #
/// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
/// # enum MyStates {
/// #     Load,
/// #     Next
/// # }
/// ```
pub struct AssetLoaderPlugin<A, T> {
    /// The state during which the assets should be loaded
    on: T,
    /// The state the plugin will set when the assets are loaded
    next: T,
    marker: PhantomData<A>,
}

impl<A, T> AssetLoaderPlugin<A, T>
where
    A: AssetCollection,
    T: Component + Debug + Clone + Eq + Hash,
{
    /// Create a new instance of [AssetLoaderPlugin]
    ///
    /// ```edition2018
    /// # use bevy_asset_loader::{AssetLoaderPlugin, AssetCollection};
    /// # use bevy::prelude::*;
    /// fn main() {
    ///    let mut app = App::build();
    ///    app
    ///         .add_plugins(DefaultPlugins)
    ///         .add_plugin(AssetLoaderPlugin::<MyAssets, _>::new(
    ///             MyStates::Load,
    ///             MyStates::Next,
    ///         ));
    ///    app.run();
    /// }
    ///
    /// # #[derive(AssetCollection)]
    /// # struct MyAssets {
    /// #     #[asset(path = "player.png")]
    /// #     player: Handle<Texture>,
    /// #     #[asset(path = "tree.png")]
    /// #     tree: Handle<Texture>
    /// # }
    /// #
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum MyStates {
    /// #     Load,
    /// #     Next
    /// # }
    /// ```
    pub fn new(on: T, next: T) -> AssetLoaderPlugin<A, T> {
        Self {
            on,
            next,
            marker: PhantomData::<A>,
        }
    }
}

/// Trait to mark a struct as a collection os assets
///
/// Derive is supported for structs with named fields of the type [bevy::asset::Handle]<T: [bevy::asset::Asset]>
/// Each field needs to be annotated with ``#[asset(path = "path/to/asset.png")]``
/// ```edition2018
/// # use bevy::asset::Handle;
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
    handles: Vec<HandleId>,
    marker: PhantomData<A>,
}

struct AssetLoaderNextState<T> {
    next: T,
}

impl<Assets, State> Plugin for AssetLoaderPlugin<Assets, State>
where
    Assets: AssetCollection,
    State: Component + Debug + Clone + Eq + Hash,
{
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(AssetLoaderNextState::<State> {
            next: self.next.clone(),
        })
        .add_system_set(
            SystemSet::on_enter(self.on.clone()).with_system(start_loading::<Assets>.system()),
        )
        .add_system_set(
            SystemSet::on_update(self.on.clone())
                .with_system(check_loading_state::<State, Assets>.system()),
        );
    }
}

fn start_loading<Assets: AssetCollection>(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut handles = Assets::load(&asset_server);
    commands.insert_resource(LoadingAssetHandles {
        handles: handles.drain(..).map(|handle| handle.id).collect(),
        marker: PhantomData::<Assets>,
    })
}

fn check_loading_state<T: Component + Debug + Clone + Eq + Hash, Assets: AssetCollection>(
    mut commands: Commands,
    mut state: ResMut<State<T>>,
    next_state: Res<AssetLoaderNextState<T>>,
    asset_server: Res<AssetServer>,
    loading_asset_handles: Res<LoadingAssetHandles<Assets>>,
) {
    let load_state = asset_server.get_group_load_state(loading_asset_handles.handles.clone());
    if load_state == LoadState::Loaded {
        commands.insert_resource(Assets::create(&asset_server));
        state
            .set(next_state.next.clone())
            .expect("Failed to set next State");
    }
}
