mod systems;

/// Configuration of loading states
pub mod config;

use bevy_app::{App, Update};
use bevy_asset::UntypedHandle;
use bevy_ecs::{
    component::Component,
    resource::Resource,
    schedule::{IntoScheduleConfigs, SystemSet},
    system::Commands,
    world::{FromWorld, World},
};
use bevy_platform::collections::HashMap;
use bevy_state::{
    condition::in_state,
    state::{FreelyMutableState, OnEnter},
};
use std::marker::PhantomData;

use crate::asset_collection::AssetCollection;
use crate::dynamic_asset::{DynamicAsset, DynamicAssets};
use crate::loading::{AssetLoadingPlugin, AssetLoadingSet, DynamicFileSpec};

use config::{ConfigureLoadingState, LoadingStateConfig};
use systems::{check_loading_coordinator, enter_loading_state};

#[cfg(feature = "standard_dynamic_assets")]
use crate::standard_dynamic_asset::{
    StandardDynamicAsset, StandardDynamicAssetArrayCollection, StandardDynamicAssetCollection,
};
#[cfg(feature = "standard_dynamic_assets")]
use bevy_common_assets::ron::RonAssetPlugin;

/// Internal dummy collection used to sequence global dynamic file loading before collections.
///
/// This type has no asset fields. It is used as a "gate": the entity-loading pipeline waits
/// for all attached dynamic file specs to finish, then the observer on this entity spawns the
/// real collection entities.
#[derive(Resource)]
pub(crate) struct DynamicPreloadFinished<S: FreelyMutableState>(PhantomData<S>);

impl<S: FreelyMutableState + 'static> AssetCollection for DynamicPreloadFinished<S> {
    fn create(_world: &mut World) -> Self {
        DynamicPreloadFinished(PhantomData)
    }
    fn load(_world: &mut World) -> Vec<UntypedHandle> {
        vec![]
    }
}

/// Marker component added to loading entities spawned by the state-based pipeline.
#[derive(Component, Default)]
pub(crate) struct LoadingForState<S: FreelyMutableState>(PhantomData<S>);

pub(crate) type CollectionSpawnerFn = Box<dyn Fn(&mut Commands) + Send + Sync>;
pub(crate) type FinallyCallbackFn = Box<dyn Fn(&mut World) + Send + Sync>;

/// Per-state-value spawner data stored by [`LoadingStateSpawners<S>`].
pub(crate) struct PerStateSpawners<S: FreelyMutableState> {
    pub(crate) global_dynamic_files: Vec<DynamicFileSpec>,
    pub(crate) collection_spawners: Vec<CollectionSpawnerFn>,
    pub(crate) finally_callbacks: Vec<FinallyCallbackFn>,
    pub(crate) next_state: Option<S>,
    pub(crate) failure_state: Option<S>,
}

impl<S: FreelyMutableState> Default for PerStateSpawners<S> {
    fn default() -> Self {
        PerStateSpawners {
            global_dynamic_files: vec![],
            collection_spawners: vec![],
            finally_callbacks: vec![],
            next_state: None,
            failure_state: None,
        }
    }
}

/// Resource storing the spawner closures and callbacks for each loading state value.
///
/// Keyed by state value so that multiple `LoadingState` configurations for different
/// variants of the same state enum coexist without conflict.
#[derive(Resource)]
pub(crate) struct LoadingStateSpawners<S: FreelyMutableState> {
    pub(crate) states: HashMap<S, PerStateSpawners<S>>,
    _marker: PhantomData<S>,
}

impl<S: FreelyMutableState> Default for LoadingStateSpawners<S> {
    fn default() -> Self {
        LoadingStateSpawners {
            states: HashMap::default(),
            _marker: PhantomData,
        }
    }
}

/// Runtime state per loading-state-value, tracking how many collection entities are still loading.
#[derive(Default, Clone)]
pub(crate) struct CoordinatorState {
    pub(crate) remaining: usize,
    pub(crate) any_failed: bool,
    pub(crate) completed: bool,
}

/// Resource storing the coordinator state for each loading state value.
#[derive(Resource)]
pub(crate) struct LoadingStateCoordinator<S: FreelyMutableState> {
    pub(crate) states: HashMap<S, CoordinatorState>,
    _marker: PhantomData<S>,
}

impl<S: FreelyMutableState> Default for LoadingStateCoordinator<S> {
    fn default() -> Self {
        LoadingStateCoordinator {
            states: HashMap::default(),
            _marker: PhantomData,
        }
    }
}

/// A Bevy plugin to configure automatic asset loading
///
/// ```edition2021
/// # use bevy_asset_loader::prelude::*;
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// # use bevy::state::app::StatesPlugin;
/// fn main() {
/// App::new()
///         .add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
///         .init_state::<GameState>()
///         .add_loading_state(LoadingState::new(GameState::Loading)
///             .continue_to_state(GameState::Menu)
///             .load_collection::<AudioAssets>()
///             .load_collection::<ImageAssets>()
///         )
///         .add_systems(OnEnter(GameState::Menu), play_audio)
/// #       .set_runner(|mut app| {app.update(); AppExit::Success})
///         .run();
/// }
///
/// fn play_audio(mut commands: Commands, audio_assets: Res<AudioAssets>) {
///     commands.spawn(AudioPlayer(audio_assets.background.clone()));
/// }
///
/// #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
/// enum GameState {
///     #[default]
///     Loading,
///     Menu
/// }
///
/// #[derive(AssetCollection, Resource)]
/// pub struct AudioAssets {
///     #[asset(path = "audio/background.ogg")]
///     pub background: Handle<AudioSource>,
/// }
///
/// #[derive(AssetCollection, Resource)]
/// pub struct ImageAssets {
///     #[asset(path = "images/player.png")]
///     pub player: Handle<Image>,
///     #[asset(path = "images/tree.png")]
///     pub tree: Handle<Image>,
/// }
/// ```
pub struct LoadingState<State: FreelyMutableState> {
    next_state: Option<State>,
    failure_state: Option<State>,
    loading_state: State,
    dynamic_assets: HashMap<String, Box<dyn DynamicAsset>>,

    #[cfg(feature = "standard_dynamic_assets")]
    standard_dynamic_asset_collection_file_endings: Vec<&'static str>,
    config: LoadingStateConfig<State>,
}

impl<S> LoadingState<S>
where
    S: FreelyMutableState,
{
    /// Create a new [`LoadingState`]
    ///
    /// This function takes a [`State`] during which all asset collections will
    /// be loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use bevy::state::app::StatesPlugin;
    /// # fn main() {
    /// App::new()
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
    /// #       .init_state::<GameState>()
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .load_collection::<AudioAssets>()
    ///             .load_collection::<ImageAssets>()
    ///         )
    /// #       .set_runner(|mut app| {app.update(); AppExit::Success})
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
    /// # enum GameState {
    /// #     #[default]
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct AudioAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct ImageAssets {
    /// #     #[asset(path = "images/player.png")]
    /// #     pub player: Handle<Image>,
    /// #     #[asset(path = "images/tree.png")]
    /// #     pub tree: Handle<Image>,
    /// # }
    /// ```
    #[must_use]
    pub fn new(load: S) -> LoadingState<S> {
        Self {
            next_state: None,
            failure_state: None,
            loading_state: load.clone(),
            dynamic_assets: HashMap::default(),
            #[cfg(feature = "standard_dynamic_assets")]
            standard_dynamic_asset_collection_file_endings: vec!["assets.ron"],
            config: LoadingStateConfig::new(load),
        }
    }

    /// The [`LoadingState`] will set this Bevy [`State`] after all asset collections
    /// are loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use bevy::state::app::StatesPlugin;
    /// # fn main() {
    /// App::new()
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
    /// #       .init_state::<GameState>()
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .load_collection::<AudioAssets>()
    ///             .load_collection::<ImageAssets>()
    ///         )
    /// #       .set_runner(|mut app| {app.update(); AppExit::Success})
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
    /// # enum GameState {
    /// #     #[default]
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct AudioAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct ImageAssets {
    /// #     #[asset(path = "images/player.png")]
    /// #     pub player: Handle<Image>,
    /// #     #[asset(path = "images/tree.png")]
    /// #     pub tree: Handle<Image>,
    /// # }
    /// ```
    #[must_use]
    pub fn continue_to_state(mut self, next: S) -> Self {
        self.next_state = Some(next);

        self
    }

    /// The [`LoadingState`] will set this Bevy [`State`] if an asset fails to load.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use bevy::state::app::StatesPlugin;
    /// # fn main() {
    /// App::new()
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
    /// #       .init_state::<GameState>()
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .on_failure_continue_to_state(GameState::Error)
    ///             .load_collection::<MyAssets>()
    ///         )
    /// #       .set_runner(|mut app| {app.update(); AppExit::Success})
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
    /// # enum GameState {
    /// #     #[default]
    /// #     Loading,
    /// #     Error,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct MyAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// ```
    #[must_use]
    pub fn on_failure_continue_to_state(mut self, next: S) -> Self {
        self.failure_state = Some(next);

        self
    }

    /// Insert a map of asset keys with corresponding standard dynamic assets
    #[must_use]
    #[cfg(feature = "standard_dynamic_assets")]
    #[cfg_attr(docsrs, doc(cfg(feature = "standard_dynamic_assets")))]
    pub fn add_standard_dynamic_assets(
        mut self,
        mut dynamic_assets: HashMap<String, StandardDynamicAsset>,
    ) -> Self {
        dynamic_assets.drain().for_each(|(key, value)| {
            self.dynamic_assets.insert(key, Box::new(value));
        });

        self
    }

    /// Set all file endings that should be loaded as [`StandardDynamicAssetCollection`].
    ///
    /// The default file ending is `.assets`
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(feature = "standard_dynamic_assets")))]
    #[cfg(feature = "standard_dynamic_assets")]
    pub fn set_standard_dynamic_asset_collection_file_endings(
        mut self,
        endings: Vec<&'static str>,
    ) -> Self {
        self.standard_dynamic_asset_collection_file_endings = endings;

        self
    }

    /// Finish configuring the [`LoadingState`]
    ///
    /// Calling this function is required to set up the asset loading.
    /// Most of the time you do not want to call this method directly though, but complete the setup
    /// using [`LoadingStateAppExt::add_loading_state`].
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use bevy::state::app::StatesPlugin;
    /// # fn main() {
    /// App::new()
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
    /// #       .init_state::<GameState>()
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .load_collection::<AudioAssets>()
    ///             .load_collection::<ImageAssets>()
    ///         )
    /// #       .set_runner(|mut app| {app.update(); AppExit::Success})
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
    /// # enum GameState {
    /// #     #[default]
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct AudioAssets {
    /// #     #[asset(path = "audio/background.ogg")]
    /// #     pub background: Handle<AudioSource>,
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct ImageAssets {
    /// #     #[asset(path = "images/player.png")]
    /// #     pub player: Handle<Image>,
    /// #     #[asset(path = "images/tree.png")]
    /// #     pub tree: Handle<Image>,
    /// # }
    /// ```
    #[allow(unused_mut)]
    pub fn build(mut self, app: &mut App) {
        // Register standard dynamic asset collection plugins before AssetLoadingPlugin
        // so AssetLoadingPlugin skips re-registering with different endings.
        #[cfg(feature = "standard_dynamic_assets")]
        if !app.is_plugin_added::<RonAssetPlugin<StandardDynamicAssetCollection>>() {
            app.add_plugins(RonAssetPlugin::<StandardDynamicAssetCollection>::new(
                &self.standard_dynamic_asset_collection_file_endings,
            ));
        }
        #[cfg(feature = "standard_dynamic_assets")]
        if !app.is_plugin_added::<RonAssetPlugin<StandardDynamicAssetArrayCollection>>() {
            app.add_plugins(RonAssetPlugin::<StandardDynamicAssetArrayCollection>::new(
                &[],
            ));
        }

        // Ensure the loading plugin is present
        if !app.is_plugin_added::<AssetLoadingPlugin>() {
            app.add_plugins(AssetLoadingPlugin);
        }

        // Pre-register any DynamicAsset entries specified via add_standard_dynamic_assets
        app.init_resource::<DynamicAssets>();
        let mut da = app.world_mut().resource_mut::<DynamicAssets>();
        for (key, asset) in self.dynamic_assets {
            da.register_asset(key, asset);
        }

        // Initialize per-S resources (idempotent)
        app.init_resource::<LoadingStateSpawners<S>>();
        app.init_resource::<LoadingStateCoordinator<S>>();

        // Store next/failure state into the per-state spawners entry
        {
            let mut spawners = app.world_mut().resource_mut::<LoadingStateSpawners<S>>();
            let per_state = spawners
                .states
                .entry(self.loading_state.clone())
                .or_default();
            if self.next_state.is_some() {
                per_state.next_state = self.next_state;
            }
            if self.failure_state.is_some() {
                per_state.failure_state = self.failure_state;
            }
        }

        // Apply collection/callback config (appends to the per-state entry)
        self.config.build(app);

        // Add the OnEnter system that spawns entities and resets the coordinator
        app.add_systems(
            OnEnter(self.loading_state.clone()),
            enter_loading_state::<S>,
        );

        // Add the coordinator check system, after the asset loading systems
        app.add_systems(
            Update,
            check_loading_coordinator::<S>
                .run_if(in_state(self.loading_state.clone()))
                .in_set(LoadingStateSet(self.loading_state.clone()))
                .after(AssetLoadingSet),
        );

        app.configure_sets(Update, LoadingStateSet(self.loading_state.clone()));
    }
}

impl<S: FreelyMutableState> ConfigureLoadingState for LoadingState<S> {
    fn load_collection<A: AssetCollection>(mut self) -> Self {
        self.config = self.config.load_collection::<A>();

        self
    }

    fn finally_init_resource<R: Resource + FromWorld>(mut self) -> Self {
        self.config = self.config.finally_init_resource::<R>();

        self
    }

    fn register_dynamic_asset_collection<
        C: crate::dynamic_asset::DynamicAssetCollection + bevy_asset::Asset,
    >(
        self,
    ) -> Self {
        self
    }

    fn with_dynamic_assets_file<
        C: crate::dynamic_asset::DynamicAssetCollection + bevy_asset::Asset,
    >(
        mut self,
        file: &str,
    ) -> Self {
        self.config = self.config.with_dynamic_assets_file::<C>(file);

        self
    }

    fn init_resource<R: Resource + FromWorld>(self) -> Self {
        self.finally_init_resource::<R>()
    }
}

///  Systems in this set check the loading state of assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct LoadingStateSet<S: FreelyMutableState>(pub S);

/// Extension trait for Bevy Apps to add loading states idiomatically
pub trait LoadingStateAppExt {
    /// Add a loading state to your app
    fn add_loading_state<S: FreelyMutableState>(
        &mut self,
        loading_state: LoadingState<S>,
    ) -> &mut Self;

    /// Configure an existing loading state with, for example, additional asset collections.
    fn configure_loading_state<S: FreelyMutableState>(
        &mut self,
        configuration: LoadingStateConfig<S>,
    ) -> &mut Self;
}

impl LoadingStateAppExt for App {
    fn add_loading_state<S: FreelyMutableState>(
        &mut self,
        loading_state: LoadingState<S>,
    ) -> &mut Self {
        loading_state.build(self);

        self
    }

    fn configure_loading_state<S: FreelyMutableState>(
        &mut self,
        configuration: LoadingStateConfig<S>,
    ) -> &mut Self {
        configuration.build(self);

        self
    }
}
