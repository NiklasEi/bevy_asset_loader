mod dynamic_asset_systems;
mod systems;

use bevy::app::{App, CoreSet, IntoSystemAppConfig, Plugin};
use bevy::asset::{Asset, HandleUntyped};
use bevy::ecs::schedule::{
    common_conditions::in_state, IntoSystemConfig, IntoSystemSetConfig, NextState, OnEnter, States,
    SystemSet,
};
use bevy::ecs::schedule::{BoxedScheduleLabel, ScheduleLabel, State};
use bevy::ecs::system::Resource;
use bevy::ecs::world::FromWorld;
use bevy::utils::{default, HashMap, HashSet};
use std::any::TypeId;
use std::marker::PhantomData;

use crate::asset_collection::AssetCollection;
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections};

use systems::{
    check_loading_collection, finish_loading_state, init_resource, initialize_loading_state,
    reset_loading_state, resume_to_finalize, start_loading_collection,
};

use dynamic_asset_systems::{
    check_dynamic_asset_collections, load_dynamic_asset_collections,
    resume_to_loading_asset_collections,
};

#[cfg(feature = "standard_dynamic_assets")]
use bevy_common_assets::ron::RonAssetPlugin;

#[cfg(feature = "standard_dynamic_assets")]
use crate::standard_dynamic_asset::{StandardDynamicAsset, StandardDynamicAssetCollection};

#[cfg(feature = "progress_tracking")]
use iyes_progress::TrackedProgressSet;

use crate::dynamic_asset::{DynamicAsset, DynamicAssets};
use crate::loading_state::systems::{apply_internal_state_transition, run_loading_state};

/// A Bevy plugin to configure automatic asset loading
///
/// ```edition2021
/// # use bevy_asset_loader::prelude::*;
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
///
/// fn main() {
///     App::new()
///         .add_state::<GameState>()
///         .add_plugins(MinimalPlugins)
/// #       .init_resource::<iyes_progress::ProgressCounter>()
///         .add_plugin(AssetPlugin::default())
///         .add_loading_state(LoadingState::new(GameState::Loading)
///             .continue_to_state(GameState::Menu)
///         )
///         .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
///         .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
///         .add_system(play_audio.in_schedule(OnEnter(GameState::Menu)))
/// #       .set_runner(|mut app| app.update())
///         .run();
/// }
///
/// fn play_audio(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
///     audio.play(audio_assets.background.clone());
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
pub struct LoadingState<State> {
    next_state: Option<State>,
    failure_state: Option<State>,
    loading_state: State,
    dynamic_assets: HashMap<String, Box<dyn DynamicAsset>>,

    #[cfg(feature = "standard_dynamic_assets")]
    standard_dynamic_asset_collection_file_endings: Vec<&'static str>,
}

impl<S> LoadingState<S>
where
    S: States,
{
    /// Create a new [`LoadingState`]
    ///
    /// This function takes a [`State`] during which all asset collections will
    /// be loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///         )
    ///         .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
    ///         .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
    /// #       .set_runner(|mut app| app.update())
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
            loading_state: load,
            dynamic_assets: HashMap::default(),
            #[cfg(feature = "standard_dynamic_assets")]
            standard_dynamic_asset_collection_file_endings: vec!["assets.ron"],
        }
    }

    /// The [`LoadingState`] will set this Bevy [`State`](::bevy::ecs::schedule::State) after all asset collections
    /// are loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///         )
    ///         .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
    ///         .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
    /// #       .set_runner(|mut app| app.update())
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

    /// The [`LoadingState`] will set this Bevy [`State`](::bevy::ecs::schedule::State) if an asset fails to load.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .on_failure_continue_to_state(GameState::Error)
    ///         )
    ///         .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading)
    /// #       .set_runner(|mut app| app.update())
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
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///         )
    ///         .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
    ///         .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
    /// #       .set_runner(|mut app| app.update())
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
        let mut asset_loader_configuration = AssetLoaderConfiguration::default();
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

        app.insert_resource(asset_loader_configuration);

        app.init_resource::<State<InternalLoadingState>>();
        app.init_resource::<NextState<InternalLoadingState>>();

        app.init_resource::<DynamicAssetCollections<S>>();
        #[cfg(feature = "standard_dynamic_assets")]
        if !app.is_plugin_added::<RonAssetPlugin<StandardDynamicAssetCollection>>() {
            app.add_plugin(RonAssetPlugin::<StandardDynamicAssetCollection>::new(
                &self.standard_dynamic_asset_collection_file_endings,
            ));
        }

        if !app.is_plugin_added::<InternalAssetLoaderPlugin<S>>() {
            app.add_plugin(InternalAssetLoaderPlugin::<S>::new());
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
            app.add_systems((
                resume_to_loading_asset_collections::<S>
                    .in_schedule(loading_state_schedule.clone())
                    .in_set(InternalLoadingStateSet::ResumeDynamicAssetCollections),
                initialize_loading_state
                    .in_schedule(loading_state_schedule.clone())
                    .in_set(InternalLoadingStateSet::Initialize),
                resume_to_finalize::<S>
                    .in_schedule(loading_state_schedule.clone())
                    .in_set(InternalLoadingStateSet::CheckAssets),
                finish_loading_state::<S>
                    .in_schedule(loading_state_schedule.clone())
                    .in_set(InternalLoadingStateSet::Finalize),
            ))
            .add_system(reset_loading_state.in_schedule(OnEnter(self.loading_state.clone())))
            .configure_set(
                LoadingStateSet(self.loading_state.clone())
                    .after(CoreSet::StateTransitions)
                    .before(CoreSet::Update)
                    .run_if(in_state(self.loading_state.clone())),
            );
            let mut loading_state_schedule = app.get_schedule_mut(loading_state_schedule).unwrap();
            loading_state_schedule
                .configure_set(
                    InternalLoadingStateSet::Initialize
                        .run_if(in_state(InternalLoadingState::Initialize)),
                )
                .configure_set(
                    InternalLoadingStateSet::CheckDynamicAssetCollections.run_if(in_state(
                        InternalLoadingState::LoadingDynamicAssetCollections,
                    )),
                )
                .configure_set(
                    InternalLoadingStateSet::ResumeDynamicAssetCollections
                        .after(InternalLoadingStateSet::CheckDynamicAssetCollections)
                        .run_if(in_state(
                            InternalLoadingState::LoadingDynamicAssetCollections,
                        )),
                )
                .configure_set(
                    InternalLoadingStateSet::CheckAssets
                        .run_if(in_state(InternalLoadingState::LoadingAssets)),
                )
                .configure_set(
                    InternalLoadingStateSet::Finalize
                        .run_if(in_state(InternalLoadingState::Finalize)),
                );

            #[cfg(feature = "progress_tracking")]
            app.add_system(
                run_loading_state::<S>
                    .in_set(TrackedProgressSet)
                    .in_base_set(LoadingStateSet(self.loading_state)),
            );
            #[cfg(not(feature = "progress_tracking"))]
            app.add_system(
                run_loading_state::<S>.in_base_set(LoadingStateSet(self.loading_state.clone())),
            );
        }

        let mut dynamic_assets = DynamicAssets::default();
        for (key, asset) in self.dynamic_assets {
            dynamic_assets.register_asset(key, asset);
        }
        app.insert_resource(dynamic_assets);
    }
}

/// This set runs after [`CoreSet::StateTransitions`] and before [`CoreSet::Update`].
/// Systems in this set check the loading state of assets and will change the [`InternalLoadingState`] accordingly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
#[system_set(base)]
pub(crate) struct LoadingStateSet<S: States>(S);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum InternalLoadingStateSet {
    Initialize,
    CheckDynamicAssetCollections,
    ResumeDynamicAssetCollections,
    CheckAssets,
    Finalize,
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct OnEnterInternalLoadingState<S: States>(pub S, pub InternalLoadingState);
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LoadingStateSchedule<S: States>(pub S);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub(crate) enum InternalLoadingState {
    /// Starting point. Here it will be decided whether or not dynamic asset collections need to be loaded.
    #[default]
    Initialize,
    /// Load dynamic asset collections and configure their key <-> asset mapping
    LoadingDynamicAssetCollections,
    /// Load the actual asset collections and check their status every frame.
    LoadingAssets,
    /// All collections are loaded and inserted. Time to e.g. run custom [insert_resource](bevy_asset_loader::AssetLoader::insert_resource).
    Finalize,
    /// A 'parking' state in case no next state is defined
    Done,
}

/// This resource is used for handles from asset collections and loading dynamic asset collection files.
/// The generic will be the [`AssetCollection`] type for the first and the [`DynamicAssetCollection`] for the second.
#[derive(Resource)]
pub(crate) struct LoadingAssetHandles<T> {
    handles: Vec<HandleUntyped>,
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

#[derive(Resource)]
pub(crate) struct AssetLoaderConfiguration<State: States> {
    state_configurations: HashMap<State, LoadingConfiguration<State>>,
}

impl<State: States> Default for AssetLoaderConfiguration<State> {
    fn default() -> Self {
        AssetLoaderConfiguration {
            state_configurations: HashMap::default(),
        }
    }
}

struct LoadingConfiguration<State: States> {
    next: Option<State>,
    failure: Option<State>,
    loading_failed: bool,
    loading_collections: usize,
    loading_dynamic_collections: HashSet<TypeId>,
}

impl<State: States> Default for LoadingConfiguration<State> {
    fn default() -> Self {
        LoadingConfiguration {
            next: None,
            failure: None,
            loading_failed: false,
            loading_collections: 0,
            loading_dynamic_collections: default(),
        }
    }
}

/// Resource to store the schedules for loading states
#[derive(Resource)]
pub struct LoadingStateSchedules<State: States> {
    /// Map to store a schedule per loading state
    pub schedules: HashMap<State, BoxedScheduleLabel>,
}

impl<State: States> Default for LoadingStateSchedules<State> {
    fn default() -> Self {
        LoadingStateSchedules {
            schedules: HashMap::default(),
        }
    }
}

/// Extension trait for Bevy Apps to add loading states idiomatically
pub trait LoadingStateAppExt {
    /// Add a loading state to your app
    fn add_loading_state<S: States>(&mut self, loading_state: LoadingState<S>) -> &mut Self;

    /// Add an [`AssetCollection`] to the [`LoadingState`]
    ///
    /// The added collection will be loaded and inserted into your Bevy app as a resource.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///         )
    ///         .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
    ///         .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
    /// #       .set_runner(|mut app| app.update())
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
    fn add_collection_to_loading_state<S: States, A: AssetCollection>(
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
    fn add_dynamic_collection_to_loading_state<S: States, C: DynamicAssetCollection + Asset>(
        &mut self,
        loading_state: S,
        file: &str,
    ) -> &mut Self;

    /// Add any [`FromWorld`] resource to be initialized after all asset collections are loaded.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .add_state::<GameState>()
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///         )
    ///         .add_collection_to_loading_state::<_, TextureForAtlas>(GameState::Loading)
    ///         .init_resource_after_loading_state::<_, TextureAtlasFromWorld>(GameState::Loading)
    /// #       .set_runner(|mut app| app.update())
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
    /// # enum GameState {
    /// #     #[default]
    /// #     Loading,
    /// #     Menu
    /// # }
    /// # #[derive(Resource)]
    /// # struct TextureAtlasFromWorld {
    /// #     atlas: Handle<TextureAtlas>
    /// # }
    /// # impl FromWorld for TextureAtlasFromWorld {
    /// #     fn from_world(world: &mut World) -> Self {
    /// #         let cell = world.cell();
    /// #         let assets = cell.get_resource::<TextureForAtlas>().expect("TextureForAtlas not loaded");
    /// #         let mut atlases = cell.get_resource_mut::<Assets<TextureAtlas>>().expect("TextureAtlases missing");
    /// #         TextureAtlasFromWorld {
    /// #             atlas: atlases.add(TextureAtlas::from_grid(assets.array.clone(), Vec2::new(250., 250.), 1, 4, None, None))
    /// #         }
    /// #     }
    /// # }
    /// # #[derive(AssetCollection, Resource)]
    /// # pub struct TextureForAtlas {
    /// #     #[asset(path = "images/female_adventurer.ogg")]
    /// #     pub array: Handle<Image>,
    /// # }
    /// ```
    fn init_resource_after_loading_state<S: States, A: Resource + FromWorld>(
        &mut self,
        loading_state: S,
    ) -> &mut Self;
}

impl LoadingStateAppExt for App {
    fn add_loading_state<S: States>(&mut self, loading_state: LoadingState<S>) -> &mut Self {
        loading_state.build(self);

        self
    }

    fn add_collection_to_loading_state<S: States, A: AssetCollection>(
        &mut self,
        loading_state: S,
    ) -> &mut Self {
        self.add_system(
            start_loading_collection::<S, A>.in_schedule(OnEnterInternalLoadingState(
                loading_state.clone(),
                InternalLoadingState::LoadingAssets,
            )),
        )
        .add_system(
            check_loading_collection::<S, A>
                .in_schedule(LoadingStateSchedule(loading_state))
                .in_set(InternalLoadingStateSet::CheckAssets),
        )
    }

    fn add_dynamic_collection_to_loading_state<S: States, C: DynamicAssetCollection + Asset>(
        &mut self,
        loading_state: S,
        file: &str,
    ) -> &mut Self {
        let mut dynamic_asset_collections = self
            .world
            .get_resource_mut::<DynamicAssetCollections<S>>()
            .unwrap();
        let mut dynamic_collections_for_state = dynamic_asset_collections
            .files
            .remove(&loading_state)
            .unwrap_or_default();
        let mut dynamic_files = dynamic_collections_for_state
            .remove(&TypeId::of::<C>())
            .unwrap_or_default();
        dynamic_files.push(file.to_owned());
        dynamic_collections_for_state.insert(TypeId::of::<C>(), dynamic_files);
        dynamic_asset_collections
            .files
            .insert(loading_state.clone(), dynamic_collections_for_state);

        self.init_resource::<LoadingAssetHandles<C>>()
            .add_system(load_dynamic_asset_collections::<S, C>.in_schedule(
                OnEnterInternalLoadingState(
                    loading_state.clone(),
                    InternalLoadingState::LoadingDynamicAssetCollections,
                ),
            ))
            .add_system(
                check_dynamic_asset_collections::<S, C>
                    .in_schedule(LoadingStateSchedule(loading_state))
                    .in_set(InternalLoadingStateSet::CheckDynamicAssetCollections),
            )
    }

    fn init_resource_after_loading_state<S: States, A: Resource + FromWorld>(
        &mut self,
        loading_state: S,
    ) -> &mut Self {
        self.add_system(init_resource::<A>.in_schedule(OnEnterInternalLoadingState(
            loading_state,
            InternalLoadingState::Finalize,
        )))
    }
}

struct InternalAssetLoaderPlugin<S> {
    _state_marker: PhantomData<S>,
}

impl<S> InternalAssetLoaderPlugin<S>
where
    S: States,
{
    fn new() -> Self {
        InternalAssetLoaderPlugin {
            _state_marker: PhantomData::default(),
        }
    }
}

impl<S> Plugin for InternalAssetLoaderPlugin<S>
where
    S: States,
{
    fn build(&self, app: &mut App) {
        app.add_system(apply_internal_state_transition::<S>.in_base_set(CoreSet::StateTransitions));
    }
}
