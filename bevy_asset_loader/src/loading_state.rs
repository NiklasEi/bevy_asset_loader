mod dynamic_asset_systems;
mod systems;

use bevy::app::{App, Plugin};
use bevy::asset::{Asset, UntypedHandle};
use bevy::ecs::schedule::SystemConfigs;
use bevy::ecs::{
    schedule::{
        common_conditions::in_state, InternedScheduleLabel, IntoSystemConfigs,
        IntoSystemSetConfigs, NextState, OnEnter, ScheduleLabel, State, States, SystemSet,
    },
    system::Resource,
    world::FromWorld,
};
use bevy::prelude::{StateTransition, Update};
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
///         .add_plugins((MinimalPlugins, AssetPlugin::default()))
/// #       .init_resource::<iyes_progress::ProgressCounter>()
///         .add_loading_state(LoadingState::new(GameState::Loading)
///             .continue_to_state(GameState::Menu)
///         )
///         .add_collection_to_loading_state::<_, AudioAssets>(GameState::Loading)
///         .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
///         .add_systems(OnEnter(GameState::Menu), play_audio)
/// #       .set_runner(|mut app| app.update())
///         .run();
/// }
///
/// fn play_audio(mut commands: Commands, audio_assets: Res<AudioAssets>) {
///     commands.spawn(AudioBundle {
///         source: audio_assets.background.clone(),
///         ..default()
///     });
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
pub struct LoadingState<State: States> {
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
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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

    /// The [`LoadingState`] will set this Bevy [`State`] if an asset fails to load.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
        app.init_resource::<AssetLoaderConfiguration<S>>();
        {
            let mut asset_loader_configuration = app
                .world
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

        app.init_resource::<DynamicAssetCollections<S>>();
        #[cfg(feature = "standard_dynamic_assets")]
        if !app.is_plugin_added::<RonAssetPlugin<StandardDynamicAssetCollection>>() {
            app.add_plugins(RonAssetPlugin::<StandardDynamicAssetCollection>::new(
                &self.standard_dynamic_asset_collection_file_endings,
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
            app.register_dynamic_asset_collection::<_, StandardDynamicAssetCollection>(
                self.loading_state.clone(),
            );

            #[cfg(feature = "progress_tracking")]
            app.add_systems(
                Update,
                run_loading_state::<S>
                    .in_set(TrackedProgressSet)
                    .in_set(LoadingStateSet(self.loading_state.clone()))
                    .run_if(in_state(self.loading_state)),
            );
            #[cfg(not(feature = "progress_tracking"))]
            app.add_systems(
                Update,
                run_loading_state::<S>
                    .in_set(LoadingStateSet(self.loading_state.clone()))
                    .run_if(in_state(self.loading_state)),
            );
        }

        app.init_resource::<DynamicAssets>();
        let mut dynamic_assets = app.world.get_resource_mut::<DynamicAssets>().unwrap();
        for (key, asset) in self.dynamic_assets {
            dynamic_assets.register_asset(key, asset);
        }
        self.config.build(app);
    }
}

impl<S: States> ConfigureLoadingState for LoadingState<S> {
    fn load_collection<A: AssetCollection>(mut self) -> Self {
        self.config = self.config.load_collection::<A>();

        self
    }

    fn init_resource<R: Resource + FromWorld>(mut self) -> Self {
        self.config = self.config.init_resource::<R>();

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
pub struct LoadingStateSet<S: States>(pub S);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum InternalLoadingStateSet {
    Initialize,
    CheckDynamicAssetCollections,
    ResumeDynamicAssetCollections,
    CheckAssets,
    Finalize,
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct OnEnterInternalLoadingState<S: States>(pub S, pub InternalLoadingState<S>);
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LoadingStateSchedule<S: States>(pub S);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub(crate) enum InternalLoadingState<S: States> {
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
    pub schedules: HashMap<State, InternedScheduleLabel>,
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

    /// Configure an existing loading state with, for example, additional asset collections.
    fn configure_loading_state<S: States>(
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
    /// # fn main() {
    ///     App::new()
    /// #       .add_state::<GameState>()
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
    #[deprecated(
        since = "0.19.0",
        note = "Use load_collection on LoadingState or LoadingStateConfig instead."
    )]
    fn add_collection_to_loading_state<S: States, A: AssetCollection>(
        &mut self,
        loading_state: S,
    ) -> &mut Self;

    /// Register a new [`DynamicAssetCollection`] to be handled in the loading state
    ///
    /// You do not need to call this for [`StandardDynamicAssetCollection`], only if you want to use
    /// your own dynamic asset collection types.
    #[deprecated(
        since = "0.19.0",
        note = "Use configure_loading_state and [`LoadingStateConfig::register_dynamic_asset_collection`] instead."
    )]
    fn register_dynamic_asset_collection<S: States, C: DynamicAssetCollection + Asset>(
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
        note = "Use configure_loading_state and [`LoadingStateConfig::with_dynamic_assets`] instead."
    )]
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
    /// #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
    /// #       .add_state::<GameState>()
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
    #[deprecated(
        since = "0.19.0",
        note = "Use configure_loading_state and [`LoadingStateConfig::init_resource`] instead."
    )]
    fn init_resource_after_loading_state<S: States, A: Resource + FromWorld>(
        &mut self,
        loading_state: S,
    ) -> &mut Self;
}

/// Can be used to add new asset collections or similar configuration to an existing loading state.
/// ```edition2021
/// # use bevy_asset_loader::prelude::*;
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// # fn main() {
///     App::new()
/// # /*
///         .add_plugins(DefaultPlugins)
/// # */
/// #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
///         .add_state::<GameState>()
/// #       .init_resource::<iyes_progress::ProgressCounter>()
///         .add_loading_state(
///           LoadingState::new(GameState::Loading)
///             .continue_to_state(GameState::Menu)
///         )
///         .configure_loading_state(LoadingStateConfig::new(GameState::Loading).load_collection::<AudioAssets>())
/// #       .set_runner(|mut app| app.update())
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
pub struct LoadingStateConfig<S: States> {
    state: S,

    on_enter_loading_assets: Vec<SystemConfigs>,
    on_enter_loading_dynamic_asset_collections: Vec<SystemConfigs>,
    on_update: Vec<SystemConfigs>,
    on_enter_finalize: Vec<SystemConfigs>,

    dynamic_assets: HashMap<TypeId, Vec<String>>,
}

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
    /// See the `init_resource` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    fn init_resource<R: Resource + FromWorld>(self) -> Self;

    /// Register a custom dynamic asset collection type
    ///
    /// See the `custom_dynamic_assets` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    fn register_dynamic_asset_collection<C: DynamicAssetCollection + Asset>(self) -> Self;

    /// Add a file containing dynamic assets to the loading state. Keys contained in the file, will
    /// be available for asset collections.
    ///
    /// See the `dynamic_asset` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    fn with_dynamic_assets_file<C: DynamicAssetCollection + Asset>(self, file: &str) -> Self;
}

impl<S: States> LoadingStateConfig<S> {
    pub fn new(state: S) -> Self {
        Self {
            state,
            on_enter_loading_assets: vec![],
            on_enter_loading_dynamic_asset_collections: vec![],
            on_update: vec![],
            on_enter_finalize: vec![],
            dynamic_assets: default(),
        }
    }

    fn with_dynamic_assets_type_id(&mut self, file: &str, type_id: TypeId) {
        let mut dynamic_files = self.dynamic_assets.remove(&type_id).unwrap_or_default();
        dynamic_files.push(file.to_owned());
        self.dynamic_assets.insert(type_id, dynamic_files);
    }

    fn build(mut self, app: &mut App) {
        for config in self.on_enter_loading_assets {
            app.add_systems(
                OnEnterInternalLoadingState(
                    self.state.clone(),
                    InternalLoadingState::LoadingAssets,
                ),
                config,
            );
        }
        for config in self.on_update {
            app.add_systems(LoadingStateSchedule(self.state.clone()), config);
        }
        for config in self.on_enter_finalize {
            app.add_systems(
                OnEnterInternalLoadingState(self.state.clone(), InternalLoadingState::Finalize),
                config,
            );
        }
        for config in self.on_enter_loading_dynamic_asset_collections {
            app.add_systems(
                OnEnterInternalLoadingState(
                    self.state.clone(),
                    InternalLoadingState::LoadingDynamicAssetCollections,
                ),
                config,
            );
        }
        let mut dynamic_assets = app
            .world
            .get_resource_mut::<DynamicAssetCollections<S>>()
            .unwrap_or_else(|| {
                panic!("Failed to get the DynamicAssetCollections resource for the loading state.")
            });
        for (id, files) in self.dynamic_assets.drain() {
            dynamic_assets.register_files_by_type_id(self.state.clone(), files, id);
        }
    }
}

impl<S: States> ConfigureLoadingState for LoadingStateConfig<S> {
    fn load_collection<A: AssetCollection>(mut self) -> Self {
        self.on_enter_loading_assets
            .push(start_loading_collection::<S, A>.into_configs());
        self.on_update.push(
            check_loading_collection::<S, A>
                .in_set(InternalLoadingStateSet::CheckAssets)
                .into_configs(),
        );

        self
    }

    fn init_resource<R: Resource + FromWorld>(mut self) -> Self {
        self.on_enter_finalize
            .push(init_resource::<R>.into_configs());

        self
    }

    fn register_dynamic_asset_collection<C: DynamicAssetCollection + Asset>(mut self) -> Self {
        self.on_enter_loading_dynamic_asset_collections
            .push(load_dynamic_asset_collections::<S, C>.into_configs());
        self.on_update.push(
            check_dynamic_asset_collections::<S, C>
                .in_set(InternalLoadingStateSet::CheckDynamicAssetCollections),
        );

        self
    }

    fn with_dynamic_assets_file<C: DynamicAssetCollection + Asset>(mut self, file: &str) -> Self {
        self.with_dynamic_assets_type_id(file, TypeId::of::<C>());

        self
    }
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
        self.add_systems(
            OnEnterInternalLoadingState(loading_state.clone(), InternalLoadingState::LoadingAssets),
            start_loading_collection::<S, A>,
        )
        .add_systems(
            LoadingStateSchedule(loading_state),
            check_loading_collection::<S, A>.in_set(InternalLoadingStateSet::CheckAssets),
        )
    }

    fn register_dynamic_asset_collection<S: States, C: DynamicAssetCollection + Asset>(
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

    fn add_dynamic_collection_to_loading_state<S: States, C: DynamicAssetCollection + Asset>(
        &mut self,
        loading_state: S,
        file: &str,
    ) -> &mut Self {
        let mut dynamic_asset_collections = self
            .world
            .get_resource_mut::<DynamicAssetCollections<S>>()
            .unwrap();

        dynamic_asset_collections.register_file::<C>(loading_state.clone(), file);
        self
    }

    fn init_resource_after_loading_state<S: States, A: Resource + FromWorld>(
        &mut self,
        loading_state: S,
    ) -> &mut Self {
        self.add_systems(
            OnEnterInternalLoadingState(loading_state, InternalLoadingState::Finalize),
            init_resource::<A>,
        )
    }

    fn configure_loading_state<S: States>(
        &mut self,
        configuration: LoadingStateConfig<S>,
    ) -> &mut Self {
        configuration.build(self);

        self
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
            _state_marker: PhantomData,
        }
    }
}

impl<S> Plugin for InternalAssetLoaderPlugin<S>
where
    S: States,
{
    fn build(&self, app: &mut App) {
        app.add_systems(StateTransition, apply_internal_state_transition::<S>);
    }
}
