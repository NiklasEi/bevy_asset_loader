use std::sync::Arc;

use crate::asset_collection::AssetCollection;
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
use crate::loading::{DynamicFileSpec, LoadCollectionCommandsExt};
use crate::loading_state::{CollectionSpawnerFn, FinallyCallbackFn, LoadingStateSpawners};
use bevy_app::App;
use bevy_asset::{Asset, AssetServer, Assets, UntypedHandle};
use bevy_ecs::{
    resource::Resource,
    system::Commands,
    world::{FromWorld, World},
};
use bevy_state::state::FreelyMutableState;

use super::systems::{on_collection_failed, on_collection_loaded};

/// Methods to configure a loading state
pub trait ConfigureLoadingState {
    /// Add the given collection to the loading state.
    ///
    /// Its loading progress will be tracked. Only when all included handles are fully loaded, the
    /// collection will be inserted to the ECS as a resource.
    ///
    /// See the `two_collections` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    fn load_collection<A: AssetCollection>(self) -> Self;

    /// The resource will be initialized at the end of the loading state using its [`FromWorld`] implementation.
    /// All asset collections will be available at that point and fully loaded.
    ///
    /// See the `finally_init_resource` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    fn finally_init_resource<R: Resource + FromWorld>(self) -> Self;

    /// Register a custom dynamic asset collection type
    ///
    /// See the `custom_dynamic_assets` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    #[deprecated(
        since = "0.25.0",
        note = "No longer needed. `with_dynamic_assets_file` handles loading and registering \
                the collection on its own. You can safely remove calls to this method."
    )]
    fn register_dynamic_asset_collection<C: DynamicAssetCollection + Asset>(self) -> Self;

    /// Add a file containing dynamic assets to the loading state. Keys contained in the file, will
    /// be available for asset collections.
    ///
    /// See the `dynamic_asset` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    fn with_dynamic_assets_file<C: DynamicAssetCollection + Asset>(self, file: &str) -> Self;

    /// The resource will be initialized at the end of the loading state using its [`FromWorld`] implementation.
    /// All asset collections will be available at that point and fully loaded.
    ///
    /// See the `finally_init_resource` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    #[deprecated(
        since = "0.22.1",
        note = "Method has been renamed to `finally_init_resource`"
    )]
    fn init_resource<R: Resource + FromWorld>(self) -> Self;
}

/// Can be used to add new asset collections or similar configuration to a loading state.
/// ```edition2021
/// # use bevy_asset_loader::prelude::*;
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// # use bevy::state::app::StatesPlugin;
/// # fn main() {
/// App::new()
/// # /*
///         .add_plugins(DefaultPlugins)
/// # */
/// #       .add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
///         .init_state::<GameState>()
///         .add_loading_state(
///           LoadingState::new(GameState::Loading)
///             .continue_to_state(GameState::Menu)
///         )
///         .configure_loading_state(LoadingStateConfig::new(GameState::Loading).load_collection::<AudioAssets>())
/// #       .set_runner(|mut app| {app.update(); AppExit::Success})
///         .run();
/// # }
///
/// # #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
/// # enum GameState {
/// #     #[default]
/// #     Loading,
/// #     Menu
/// # }
/// # #[derive(AssetCollection, Resource)]
/// # struct AudioAssets {
/// #     #[asset(path = "audio/background.ogg")]
/// #     background: Handle<AudioSource>,
/// #     #[asset(path = "audio/plop.ogg")]
/// #     plop: Handle<AudioSource>
/// # }
/// ```
pub struct LoadingStateConfig<S: FreelyMutableState> {
    state: S,
    collection_spawners: Vec<CollectionSpawnerFn>,
    finally_callbacks: Vec<FinallyCallbackFn>,
    global_dynamic_file_specs: Vec<DynamicFileSpec>,
}

impl<S: FreelyMutableState> LoadingStateConfig<S> {
    /// Create a new configuration for the given loading state
    pub fn new(state: S) -> Self {
        Self {
            state,
            collection_spawners: vec![],
            finally_callbacks: vec![],
            global_dynamic_file_specs: vec![],
        }
    }

    pub(crate) fn build(self, app: &mut App) {
        let mut spawners = app
            .world_mut()
            .get_resource_mut::<LoadingStateSpawners<S>>()
            .unwrap_or_else(|| {
                panic!(
                    "Failed to get the LoadingStateSpawners resource for the loading state \
                     '{:?}'. Are you trying to configure a loading state before it was added \
                     to the bevy App?",
                    self.state
                )
            });
        let per_state = spawners.states.entry(self.state).or_default();
        per_state
            .collection_spawners
            .extend(self.collection_spawners);
        per_state.finally_callbacks.extend(self.finally_callbacks);
        per_state
            .global_dynamic_files
            .extend(self.global_dynamic_file_specs);
    }
}

impl<S: FreelyMutableState> ConfigureLoadingState for LoadingStateConfig<S> {
    fn load_collection<A: AssetCollection>(mut self) -> Self {
        self.collection_spawners
            .push(Box::new(|commands: &mut Commands| {
                commands
                    .load_collection::<A>()
                    .observe(on_collection_loaded::<S, A>)
                    .observe(on_collection_failed::<S, A>);
            }));
        self
    }

    fn finally_init_resource<R: Resource + FromWorld>(mut self) -> Self {
        self.finally_callbacks.push(Box::new(|world: &mut World| {
            let _ = world.init_resource::<R>();
        }));
        self
    }

    fn register_dynamic_asset_collection<C: DynamicAssetCollection + Asset>(self) -> Self {
        // The user registers the asset plugin for C themselves (e.g. RonAssetPlugin).
        // The DynamicFileSpec in with_dynamic_assets_file handles loading and registering.
        // This method is a no-op.
        self
    }

    fn with_dynamic_assets_file<C: DynamicAssetCollection + Asset>(mut self, file: &str) -> Self {
        let path = file.to_string();
        self.global_dynamic_file_specs.push(DynamicFileSpec {
            load_fn: Arc::new(move |asset_server: &AssetServer| {
                asset_server.load::<C>(path.clone()).untyped()
            }),
            register_fn: Arc::new(move |handle: UntypedHandle, world: &mut World| {
                let typed_handle = handle.typed::<C>();
                let mut dynamic_assets =
                    world.remove_resource::<DynamicAssets>().unwrap_or_default();
                {
                    let assets = world.resource::<Assets<C>>();
                    if let Some(collection) = assets.get(&typed_handle) {
                        collection.register(&mut dynamic_assets);
                    }
                }
                world.insert_resource(dynamic_assets);
            }),
        });
        self
    }

    fn init_resource<R: Resource + FromWorld>(self) -> Self {
        self.finally_init_resource::<R>()
    }
}
