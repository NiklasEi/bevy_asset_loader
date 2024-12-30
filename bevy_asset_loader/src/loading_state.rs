mod dynamic_asset_systems;
mod systems;

/// Configuration of loading states
pub mod config;

use bevy::app::{App, Plugin};
use bevy::asset::{Asset, UntypedHandle};
use bevy::ecs::{
    schedule::{
        InternedScheduleLabel, IntoSystemConfigs, IntoSystemSetConfigs, ScheduleLabel, SystemSet,
    },
    system::Resource,
    world::FromWorld,
};
use bevy::prelude::{in_state, NextState, OnEnter, State, StateTransition, States, Update};
use bevy::state::state::FreelyMutableState;
use bevy::utils::{default, HashMap, HashSet};
use std::any::TypeId;
use std::marker::PhantomData;

use crate::asset_collection::AssetCollection;
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections};

use config::{ConfigureLoadingState, LoadingStateConfig};
use dynamic_asset_systems::{
    check_dynamic_asset_collections, load_dynamic_asset_collections,
    resume_to_loading_asset_collections,
};
use systems::{
    check_loading_collection, finally_init_resource, finish_loading_state,
    initialize_loading_state, reset_loading_state, resume_to_finalize, start_loading_collection,
};

#[cfg(feature = "standard_dynamic_assets")]
use crate::standard_dynamic_asset::{
    StandardDynamicAsset, StandardDynamicAssetArrayCollection, StandardDynamicAssetCollection,
};
#[cfg(feature = "standard_dynamic_assets")]
use bevy_common_assets::ron::RonAssetPlugin;
#[cfg(feature = "progress_tracking")]
use iyes_progress::ProgressEntryId;

use crate::dynamic_asset::{DynamicAsset, DynamicAssets};
use crate::loading_state::systems::{apply_internal_state_transition, run_loading_state};

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
        app.init_resource::<AssetLoaderConfiguration<S>>();
        {
            let mut asset_loader_configuration = app
                .world_mut()
                .get_resource_mut::<AssetLoaderConfiguration<S>>()
                .unwrap();
            let mut loading_config = asset_loader_configuration
                .state_configurations
                .remove(&self.loading_state)
                .unwrap_or_default();
            if self.next_state.is_some() {
                loading_config.next = self.next_state;
            }
            if self.failure_state.is_some() {
                loading_config.failure = self.failure_state;
            }
            asset_loader_configuration
                .state_configurations
                .insert(self.loading_state.clone(), loading_config);
        }
        app.init_resource::<State<InternalLoadingState<S>>>();
        app.init_resource::<NextState<InternalLoadingState<S>>>();
        #[cfg(feature = "progress_tracking")]
        app.insert_resource(LoadingStateProgressId::<S> {
            id: ProgressEntryId::new(),
            _marker: default(),
        });

        app.init_resource::<DynamicAssetCollections<S>>();
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

        if !app.is_plugin_added::<InternalAssetLoaderPlugin<S>>() {
            app.add_plugins(InternalAssetLoaderPlugin::<S>::new());
        }

        app.init_resource::<LoadingStateSchedules<S>>();

        let loading_state_schedule = LoadingStateSchedule(self.loading_state.clone());
        let configure_loading_state = app.get_schedule(loading_state_schedule.clone()).is_none();
        app.init_schedule(loading_state_schedule.clone())
            .init_schedule(OnEnterInternalLoadingState(
                self.loading_state.clone(),
                InternalLoadingState::LoadingAssets,
            ))
            .init_schedule(OnEnterInternalLoadingState(
                self.loading_state.clone(),
                InternalLoadingState::LoadingDynamicAssetCollections,
            ))
            .init_schedule(OnEnterInternalLoadingState(
                self.loading_state.clone(),
                InternalLoadingState::Finalize,
            ));

        if configure_loading_state {
            app.add_systems(
                loading_state_schedule.clone(),
                (
                    resume_to_loading_asset_collections::<S>
                        .in_set(InternalLoadingStateSet::ResumeDynamicAssetCollections),
                    initialize_loading_state::<S>.in_set(InternalLoadingStateSet::Initialize),
                    resume_to_finalize::<S>.in_set(InternalLoadingStateSet::CheckAssets),
                    finish_loading_state::<S>.in_set(InternalLoadingStateSet::Finalize),
                ),
            )
            .add_systems(
                OnEnter(self.loading_state.clone()),
                reset_loading_state::<S>,
            )
            .configure_sets(Update, LoadingStateSet(self.loading_state.clone()));
            let mut loading_state_schedule = app.get_schedule_mut(loading_state_schedule).unwrap();
            loading_state_schedule
                .configure_sets(
                    InternalLoadingStateSet::Initialize
                        .run_if(in_state(InternalLoadingState::<S>::Initialize)),
                )
                .configure_sets(
                    InternalLoadingStateSet::CheckDynamicAssetCollections.run_if(in_state(
                        InternalLoadingState::<S>::LoadingDynamicAssetCollections,
                    )),
                )
                .configure_sets(
                    InternalLoadingStateSet::ResumeDynamicAssetCollections
                        .after(InternalLoadingStateSet::CheckDynamicAssetCollections)
                        .run_if(in_state(
                            InternalLoadingState::<S>::LoadingDynamicAssetCollections,
                        )),
                )
                .configure_sets(
                    InternalLoadingStateSet::CheckAssets
                        .run_if(in_state(InternalLoadingState::<S>::LoadingAssets)),
                )
                .configure_sets(
                    InternalLoadingStateSet::Finalize
                        .run_if(in_state(InternalLoadingState::<S>::Finalize)),
                );

            #[cfg(feature = "standard_dynamic_assets")]
            {
                self.config = self
                    .config
                    .register_dynamic_asset_collection::<StandardDynamicAssetCollection>();
                self.config = self
                    .config
                    .register_dynamic_asset_collection::<StandardDynamicAssetArrayCollection>();
            }

            app.add_systems(
                Update,
                run_loading_state::<S>
                    .in_set(LoadingStateSet(self.loading_state.clone()))
                    .run_if(in_state(self.loading_state)),
            );
        }

        app.init_resource::<DynamicAssets>();
        let mut dynamic_assets = app.world_mut().get_resource_mut::<DynamicAssets>().unwrap();
        for (key, asset) in self.dynamic_assets {
            dynamic_assets.register_asset(key, asset);
        }
        self.config.build(app);
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

    fn register_dynamic_asset_collection<C: DynamicAssetCollection + Asset>(mut self) -> Self {
        self.config = self.config.register_dynamic_asset_collection::<C>();

        self
    }

    fn with_dynamic_assets_file<C: DynamicAssetCollection + Asset>(mut self, file: &str) -> Self {
        self.config
            .with_dynamic_assets_type_id(file, TypeId::of::<C>());

        self
    }
}

///  Systems in this set check the loading state of assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct LoadingStateSet<S: FreelyMutableState>(pub S);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum InternalLoadingStateSet {
    Initialize,
    CheckDynamicAssetCollections,
    ResumeDynamicAssetCollections,
    CheckAssets,
    Finalize,
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct OnEnterInternalLoadingState<S: FreelyMutableState>(
    pub S,
    pub InternalLoadingState<S>,
);
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LoadingStateSchedule<S: FreelyMutableState>(pub S);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub(crate) enum InternalLoadingState<S: FreelyMutableState> {
    /// Starting point. Here it will be decided whether dynamic asset collections need to be loaded.
    #[default]
    Initialize,
    /// Load dynamic asset collections and configure their key <-> asset mapping
    LoadingDynamicAssetCollections,
    /// Load the actual asset collections and check their status every frame.
    LoadingAssets,
    /// All collections are loaded and inserted. Time to e.g. run custom [`insert_resource`](bevy_asset_loader::AssetLoader::insert_resource).
    Finalize,
    /// A 'parking' state in case no next state is defined
    Done(PhantomData<S>),
}

/// This resource is used for handles from asset collections and loading dynamic asset collection files.
/// The generic will be the [`AssetCollection`] type for the first and the [`DynamicAssetCollection`] for the second.
#[derive(Resource)]
pub(crate) struct LoadingAssetHandles<T> {
    handles: Vec<UntypedHandle>,
    marker: PhantomData<T>,
}

impl<T> Default for LoadingAssetHandles<T> {
    fn default() -> Self {
        LoadingAssetHandles {
            handles: Default::default(),
            marker: Default::default(),
        }
    }
}

#[cfg(feature = "progress_tracking")]
#[derive(Resource)]
struct LoadingStateProgressId<State: FreelyMutableState> {
    id: ProgressEntryId,
    _marker: PhantomData<State>,
}

#[cfg(feature = "progress_tracking")]
#[derive(Resource)]
struct AssetCollectionsProgressId<State: FreelyMutableState, Assets: AssetCollection> {
    id: ProgressEntryId,
    _marker_state: PhantomData<State>,
    _marker_assets: PhantomData<Assets>,
}

#[cfg(feature = "progress_tracking")]
impl<State: FreelyMutableState, Assets: AssetCollection> AssetCollectionsProgressId<State, Assets> {
    pub(crate) fn new(id: ProgressEntryId) -> Self {
        AssetCollectionsProgressId {
            id,
            _marker_state: default(),
            _marker_assets: default(),
        }
    }
}

#[derive(Resource)]
pub(crate) struct AssetLoaderConfiguration<State: FreelyMutableState> {
    state_configurations: HashMap<State, LoadingConfiguration<State>>,
}

impl<State: FreelyMutableState> Default for AssetLoaderConfiguration<State> {
    fn default() -> Self {
        AssetLoaderConfiguration {
            state_configurations: HashMap::default(),
        }
    }
}

struct LoadingConfiguration<State: FreelyMutableState> {
    next: Option<State>,
    failure: Option<State>,
    loading_failed: bool,
    loading_collections: HashSet<TypeId>,
    loading_dynamic_collections: HashSet<TypeId>,
}

impl<State: FreelyMutableState> Default for LoadingConfiguration<State> {
    fn default() -> Self {
        LoadingConfiguration {
            next: None,
            failure: None,
            loading_failed: false,
            loading_collections: default(),
            loading_dynamic_collections: default(),
        }
    }
}

/// Resource to store the schedules for loading states
#[derive(Resource)]
pub struct LoadingStateSchedules<State: FreelyMutableState> {
    /// Map to store a schedule per loading state
    pub schedules: HashMap<State, InternedScheduleLabel>,
}

impl<State: FreelyMutableState> Default for LoadingStateSchedules<State> {
    fn default() -> Self {
        LoadingStateSchedules {
            schedules: HashMap::default(),
        }
    }
}

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

    /// Add an [`AssetCollection`] to the [`LoadingState`]
    ///
    /// The added collection will be loaded and inserted into your Bevy app as a resource.
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
    #[deprecated(
        since = "0.19.0",
        note = "Use `LoadingStateConfig::load_collection` or `LoadingState::load_collection` instead."
    )]
    fn add_collection_to_loading_state<S: FreelyMutableState, A: AssetCollection>(
        &mut self,
        loading_state: S,
    ) -> &mut Self;

    /// Register a new [`DynamicAssetCollection`] to be handled in the loading state
    ///
    /// You do not need to call this for [`StandardDynamicAssetCollection`], only if you want to use
    /// your own dynamic asset collection types.
    #[deprecated(
        since = "0.19.0",
        note = "Use `LoadingStateConfig::register_dynamic_asset_collection` or `LoadingState::register_dynamic_asset_collection` instead."
    )]
    fn register_dynamic_asset_collection<S: FreelyMutableState, C: DynamicAssetCollection + Asset>(
        &mut self,
        loading_state: S,
    ) -> &mut Self;

    /// Register files to be loaded as a certain type of [`DynamicAssetCollection`]
    ///
    /// During the loading state, the given dynamic asset collections will be loaded and their
    /// content registered. This will happen before trying to resolve any dynamic assets
    /// as part of asset collections.
    ///
    /// You need to register a loader for your asset type yourself.
    /// If you want to see some code, take a look at the `custom_dynamic_assets` example.
    #[deprecated(
        since = "0.19.0",
        note = "Use `LoadingState::with_dynamic_assets_file` or `LoadingStateConfig::with_dynamic_assets_file` instead."
    )]
    fn add_dynamic_collection_to_loading_state<
        S: FreelyMutableState,
        C: DynamicAssetCollection + Asset,
    >(
        &mut self,
        loading_state: S,
        file: &str,
    ) -> &mut Self;

    /// Add any [`FromWorld`] resource to be initialized after all asset collections are loaded.
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
    ///             .load_collection::<TextureForAtlas>()
    ///             .finally_init_resource::<TextureAtlasLayoutFromWorld>()
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
    /// # #[derive(Resource)]
    /// # struct TextureAtlasLayoutFromWorld {
    /// #     atlas_layout: Handle<TextureAtlasLayout>
    /// # }
    /// # impl FromWorld for TextureAtlasLayoutFromWorld {
    /// #     fn from_world(world: &mut World) -> Self {
    /// #         let mut layouts = world.get_resource_mut::<Assets<TextureAtlasLayout>>().expect("TextureAtlasLayouts missing");
    /// #         TextureAtlasLayoutFromWorld {
    /// #             atlas_layout: layouts.add(TextureAtlasLayout::from_grid(UVec2::new(250, 250), 1, 4, None, None))
    /// #         }
    /// #     }
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct TextureForAtlas {
    /// #     #[asset(path = "images/female_adventurer.ogg")]
    /// #     pub array: Handle<Image>,
    /// # }
    /// ```
    #[deprecated(
        since = "0.19.0",
        note = "Use `LoadingState::finally_init_resource` or `LoadingStateConfig::finally_init_resource` instead."
    )]
    fn init_resource_after_loading_state<S: FreelyMutableState, A: Resource + FromWorld>(
        &mut self,
        loading_state: S,
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

    fn add_collection_to_loading_state<S: FreelyMutableState, A: AssetCollection>(
        &mut self,
        loading_state: S,
    ) -> &mut Self {
        self.add_systems(
            OnEnterInternalLoadingState(loading_state.clone(), InternalLoadingState::LoadingAssets),
            start_loading_collection::<S, A>,
        )
        .add_systems(
            LoadingStateSchedule(loading_state),
            check_loading_collection::<S, A>.in_set(InternalLoadingStateSet::CheckAssets),
        )
    }

    fn register_dynamic_asset_collection<
        S: FreelyMutableState,
        C: DynamicAssetCollection + Asset,
    >(
        &mut self,
        loading_state: S,
    ) -> &mut Self {
        self.add_systems(
            OnEnterInternalLoadingState(
                loading_state.clone(),
                InternalLoadingState::LoadingDynamicAssetCollections,
            ),
            load_dynamic_asset_collections::<S, C>,
        )
        .add_systems(
            LoadingStateSchedule(loading_state),
            check_dynamic_asset_collections::<S, C>
                .in_set(InternalLoadingStateSet::CheckDynamicAssetCollections),
        )
    }

    fn add_dynamic_collection_to_loading_state<
        S: FreelyMutableState,
        C: DynamicAssetCollection + Asset,
    >(
        &mut self,
        loading_state: S,
        file: &str,
    ) -> &mut Self {
        let mut dynamic_asset_collections = self
            .world_mut()
            .get_resource_mut::<DynamicAssetCollections<S>>()
            .unwrap();

        dynamic_asset_collections.register_file::<C>(loading_state.clone(), file);
        self
    }

    fn init_resource_after_loading_state<S: FreelyMutableState, A: Resource + FromWorld>(
        &mut self,
        loading_state: S,
    ) -> &mut Self {
        self.add_systems(
            OnEnterInternalLoadingState(loading_state, InternalLoadingState::Finalize),
            finally_init_resource::<A>,
        )
    }
}

struct InternalAssetLoaderPlugin<S> {
    _state_marker: PhantomData<S>,
}

impl<S> InternalAssetLoaderPlugin<S>
where
    S: FreelyMutableState,
{
    fn new() -> Self {
        InternalAssetLoaderPlugin {
            _state_marker: PhantomData,
        }
    }
}

impl<S> Plugin for InternalAssetLoaderPlugin<S>
where
    S: FreelyMutableState,
{
    fn build(&self, app: &mut App) {
        app.add_systems(StateTransition, apply_internal_state_transition::<S>);
    }
}
