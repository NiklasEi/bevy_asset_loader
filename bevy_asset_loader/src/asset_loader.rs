mod dynamic_asset;
mod dynamic_asset_systems;
mod systems;

use bevy::app::App;
use bevy::asset::HandleUntyped;
use bevy::ecs::schedule::{
    ExclusiveSystemDescriptorCoercion, Schedule, State, StateData, SystemSet, SystemStage,
};
use bevy::ecs::system::IntoExclusiveSystem;
use bevy::ecs::world::FromWorld;
use bevy::utils::HashMap;
use std::marker::PhantomData;

use crate::asset_collection::AssetCollection;
use systems::{
    finish_loading_state, initialize_loading_state, reset_loading_state, run_loading_state,
};

#[cfg(feature = "dynamic_assets")]
use bevy::asset::Handle;
#[cfg(feature = "dynamic_assets")]
use bevy_asset_ron::RonAssetPlugin;
#[cfg(feature = "dynamic_assets")]
use dynamic_asset::DynamicAssetCollection;

#[cfg(feature = "progress_tracking")]
use iyes_progress::ProgressSystemLabel;

pub use dynamic_asset::{DynamicAsset, DynamicAssets};

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
/// #       .init_resource::<iyes_progress::ProgressCounter>()
///         .add_plugin(AssetPlugin::default());
///     AssetLoader::new(GameState::Loading)
///         .continue_to_state(GameState::Menu)
///         .with_collection::<AudioAssets>()
///         .with_collection::<ImageAssets>()
///         .build(&mut app);
///
///     app.add_state(GameState::Loading)
///         .add_system_set(SystemSet::on_enter(GameState::Menu)
///             .with_system(play_audio)
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
    dynamic_assets: HashMap<String, DynamicAsset>,
    collection_count: usize,
    start_loading_assets: SystemSet,
    check_loading_assets: SystemSet,
    initialize_resources: SystemSet,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_file_ending: &'static str,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_files: Vec<String>,
}

impl<S> AssetLoader<S>
where
    S: StateData,
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
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
    pub fn new(load: S) -> AssetLoader<S> {
        Self {
            next_state: None,
            loading_state: load,
            dynamic_assets: HashMap::default(),
            collection_count: 0,
            start_loading_assets: SystemSet::on_enter(LoadingState::LoadingAssets),
            check_loading_assets: SystemSet::on_update(LoadingState::LoadingAssets),
            initialize_resources: SystemSet::on_enter(LoadingState::Finalize),
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
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
    pub fn continue_to_state(mut self, next: S) -> Self {
        self.next_state = Some(next);

        self
    }

    /// Register an asset collection file to be loaded and used to define dynamic assets.
    ///
    /// The file will be loaded as [`DynamicAssetCollection`](crate::dynamic_asset::DynamicAssetCollection).
    /// It's mapping of asset keys to dynamic assets will be used during the loading state to resolve asset keys.
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
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
        self.start_loading_assets = self
            .start_loading_assets
            .with_system(systems::start_loading_collection::<S, A>.exclusive_system());
        self.check_loading_assets = self
            .check_loading_assets
            .with_system(systems::check_loading_collection::<S, A>.exclusive_system());
        self.collection_count += 1;

        self
    }

    /// Insert a map of asset keys with corresponding dynamic assets
    pub fn add_dynamic_assets(mut self, mut dynamic_assets: HashMap<String, DynamicAsset>) -> Self {
        dynamic_assets.drain().for_each(|(key, value)| {
            self.dynamic_assets.insert(key, value);
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
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
        self.initialize_resources = self
            .initialize_resources
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
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
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
        if !app.world.contains_resource::<AssetLoaderConfiguration<S>>() {
            app.world
                .insert_resource(AssetLoaderConfiguration::<S>::default());
        }
        if !app.world.contains_resource::<LoadingStateSchedules<S>>() {
            app.world
                .insert_resource(LoadingStateSchedules::<S>::default());
        }
        let config = LoadingConfiguration {
            next: self.next_state.clone(),
            count: 0,
        };
        {
            let mut asset_loader_configuration = app
                .world
                .get_resource_mut::<AssetLoaderConfiguration<S>>()
                .unwrap();
            asset_loader_configuration
                .configuration
                .insert(self.loading_state.clone(), config);
            #[cfg(feature = "dynamic_assets")]
            asset_loader_configuration
                .asset_collection_files
                .insert(self.loading_state.clone(), self.asset_collection_files);
        }

        let mut loading_schedule = Schedule::default();
        let mut update = SystemStage::parallel();

        #[cfg(feature = "dynamic_assets")]
        {
            app.add_plugin(RonAssetPlugin::<DynamicAssetCollection>::new(&[
                self.asset_collection_file_ending
            ]));

            update.add_system_set(
                SystemSet::on_enter(LoadingState::LoadingDynamicAssetCollections).with_system(
                    dynamic_asset_systems::load_dynamic_asset_collections::<S>.exclusive_system(),
                ),
            );
            update.add_system_set(
                SystemSet::on_update(LoadingState::LoadingDynamicAssetCollections).with_system(
                    dynamic_asset_systems::check_dynamic_asset_collections::<S>.exclusive_system(),
                ),
            );
        }
        app.insert_resource(DynamicAssets {
            key_asset_map: self.dynamic_assets,
        });

        update.add_system_set(
            SystemSet::on_update(LoadingState::Initialize).with_system(initialize_loading_state),
        );
        update.add_system_set(self.start_loading_assets);
        update.add_system_set(self.check_loading_assets);
        update.add_system_set(self.initialize_resources);
        update.add_system_set(
            SystemSet::on_update(LoadingState::Finalize).with_system(finish_loading_state::<S>),
        );

        loading_schedule.add_stage("update", update);

        app.insert_resource(State::new(LoadingState::Initialize));
        loading_schedule.add_system_set_to_stage("update", State::<LoadingState>::get_driver());

        let mut loading_state_schedules = app
            .world
            .get_resource_mut::<LoadingStateSchedules<S>>()
            .unwrap();
        loading_state_schedules
            .schedules
            .insert(self.loading_state.clone(), loading_schedule);

        app.add_system_set(
            SystemSet::on_enter(self.loading_state.clone()).with_system(reset_loading_state),
        );

        #[cfg(feature = "progress_tracking")]
        let loading_state_system = run_loading_state::<S>
            .exclusive_system()
            .at_start()
            .after(ProgressSystemLabel::Preparation);
        #[cfg(not(feature = "progress_tracking"))]
        let loading_state_system = run_loading_state::<S>.exclusive_system().at_start();

        app.add_system_set(
            SystemSet::on_update(self.loading_state).with_system(loading_state_system),
        );
    }
}

struct LoadingAssetHandles<A: AssetCollection> {
    handles: Vec<HandleUntyped>,
    marker: PhantomData<A>,
}

pub(crate) struct AssetLoaderConfiguration<State> {
    configuration: HashMap<State, LoadingConfiguration<State>>,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_handles: Vec<Handle<DynamicAssetCollection>>,
    #[cfg(feature = "dynamic_assets")]
    asset_collection_files: HashMap<State, Vec<String>>,
}

impl<State> Default for AssetLoaderConfiguration<State> {
    fn default() -> Self {
        AssetLoaderConfiguration {
            configuration: HashMap::default(),
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LoadingState {
    /// Starting point. Here it will be decided whether or not dynamic asset collections need to be loaded.
    Initialize,
    /// Load dynamic asset collections and configure their key <-> asset mapping
    #[cfg(feature = "dynamic_assets")]
    LoadingDynamicAssetCollections,
    /// Load the actual asset collections and check their status every frame.
    LoadingAssets,
    /// All collections are loaded and inserted. Time to e.g. run custom [insert_resource](bevy_asset_loader::AssetLoader::insert_resource).
    Finalize,
    /// A 'parking' state in case no next state is defined
    Done,
}

struct LoadingConfiguration<T> {
    next: Option<T>,
    count: usize,
}

/// Resource to store the schedules for loading states
pub struct LoadingStateSchedules<State: StateData> {
    /// Map to store a schedule per loading state
    pub schedules: HashMap<State, Schedule>,
}

impl<State: StateData> Default for LoadingStateSchedules<State> {
    fn default() -> Self {
        LoadingStateSchedules {
            schedules: HashMap::default(),
        }
    }
}
