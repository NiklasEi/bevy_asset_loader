mod dynamic_asset;
#[cfg(all(feature = "dynamic_assets", not(feature = "stageless")))]
mod dynamic_asset_systems;

#[cfg(not(feature = "stageless"))]
mod systems;

#[cfg(feature = "stageless")]
mod stageless;

use bevy::app::App;
use bevy::asset::HandleUntyped;
#[cfg(not(feature = "stageless"))]
use bevy::ecs::schedule::State;
use bevy::ecs::schedule::{
    ExclusiveSystemDescriptorCoercion, Schedule, StateData, SystemSet, SystemStage,
};
use bevy::ecs::system::IntoExclusiveSystem;
use bevy::ecs::world::FromWorld;
#[cfg(feature = "stageless")]
use bevy::prelude::CoreStage;
use bevy::utils::HashMap;
use std::marker::PhantomData;

use crate::asset_collection::AssetCollection;
#[cfg(feature = "stageless")]
use stageless::systems::{
    check_loading_collection, finish_loading_state, init_resource, initialize_loading_state,
    reset_loading_state, run_loading_state, start_loading_collection,
};
#[cfg(not(feature = "stageless"))]
use systems::{
    check_loading_collection, finish_loading_state, init_resource, initialize_loading_state,
    reset_loading_state, run_loading_state, start_loading_collection,
};

#[cfg(all(feature = "dynamic_assets", not(feature = "stageless")))]
use dynamic_asset_systems::{check_dynamic_asset_collections, load_dynamic_asset_collections};

#[cfg(all(feature = "dynamic_assets", feature = "stageless"))]
use stageless::dynamic_asset_systems::{
    check_dynamic_asset_collections, load_dynamic_asset_collections,
};

#[cfg(feature = "dynamic_assets")]
use bevy_common_assets::ron::RonAssetPlugin;

#[cfg(feature = "dynamic_assets")]
pub use dynamic_asset::{DynamicAssetCollections, StandardDynamicAssetCollection};

#[cfg(feature = "progress_tracking")]
use iyes_progress::ProgressSystemLabel;

#[cfg(feature = "stageless")]
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};

#[cfg(feature = "stageless")]
use iyes_loopless::state::app::StateTransitionStageLabel;

#[cfg(feature = "stageless")]
use iyes_loopless::state::StateTransitionStage;

pub use dynamic_asset::{
    DynamicAsset, DynamicAssetCollection, DynamicAssetType, DynamicAssets, StandardDynamicAsset,
};

/// A Bevy plugin to configure automatic asset loading
///
/// ```edition2021
/// # use bevy_asset_loader::{AssetLoader, AssetCollection};
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// # #[cfg(feature="stageless")]
/// # use iyes_loopless::prelude::*;
///
/// # #[cfg(not(feature="stageless"))]
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
/// # #[cfg(feature="stageless")]
/// # fn main() {
/// #    let mut app = App::new();
/// #    app
/// #        .add_loopless_state(GameState::Loading)
/// #        .add_plugins(MinimalPlugins)
/// #       .init_resource::<iyes_progress::ProgressCounter>()
/// #        .add_plugin(AssetPlugin::default());
/// #    AssetLoader::new(GameState::Loading)
/// #        .continue_to_state(GameState::Menu)
/// #        .with_collection::<AudioAssets>()
/// #        .with_collection::<ImageAssets>()
/// #        .build(&mut app);
/// #
/// #   app
/// #       .add_enter_system(GameState::Menu, play_audio)
/// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
/// #        .run();
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
    dynamic_assets: HashMap<String, Box<dyn DynamicAsset>>,
    collection_count: usize,
    check_loading_assets: SystemSet,
    #[cfg(not(feature = "stageless"))]
    start_loading_assets: SystemSet,
    #[cfg(not(feature = "stageless"))]
    initialize_resources: SystemSet,

    #[cfg(feature = "dynamic_assets")]
    dynamic_asset_collection_file_endings: Vec<&'static str>,
    #[cfg(feature = "dynamic_assets")]
    dynamic_asset_collections: Vec<String>,

    #[cfg(feature = "stageless")]
    loading_transition_stage: StateTransitionStage<LoadingState>,
}

impl<S> AssetLoader<S>
where
    S: StateData,
{
    /// Create a new [`AssetLoader`]
    ///
    /// This function takes a [`State`] during which all asset collections will
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
    #[must_use]
    #[cfg(not(feature = "stageless"))]
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
            dynamic_asset_collection_file_endings: vec!["assets"],
            #[cfg(feature = "dynamic_assets")]
            dynamic_asset_collections: vec![],
        }
    }

    /// Create a new [`AssetLoader`]
    ///
    /// This function takes a [`State`](bevy_ecs::schedule::State) during which all asset collections will
    /// be loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<ImageAssets>()
    ///         .build(&mut app);
    /// #   app
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
    #[must_use]
    #[cfg(feature = "stageless")]
    pub fn new(load: S) -> Self {
        Self {
            next_state: None,
            loading_state: load,
            dynamic_assets: HashMap::default(),
            collection_count: 0,
            check_loading_assets: ConditionSet::new()
                .run_in_state(LoadingState::LoadingAssets)
                .into(),
            loading_transition_stage: StateTransitionStage::new(LoadingState::Initialize),
            #[cfg(feature = "dynamic_assets")]
            dynamic_asset_collection_file_endings: vec!["assets"],
            #[cfg(feature = "dynamic_assets")]
            dynamic_asset_collections: vec![],
        }
    }

    /// The [`AssetLoader`] will set this [`State`] after all asset collections
    /// are loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_loopless_state(GameState::Loading)
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
    #[must_use]
    pub fn continue_to_state(mut self, next: S) -> Self {
        self.next_state = Some(next);

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
    #[must_use]
    #[cfg(not(feature = "stageless"))]
    pub fn with_collection<A: AssetCollection>(mut self) -> Self {
        self.start_loading_assets = self
            .start_loading_assets
            .with_system(start_loading_collection::<S, A>.exclusive_system());
        self.check_loading_assets = self
            .check_loading_assets
            .with_system(check_loading_collection::<S, A>.exclusive_system());
        self.collection_count += 1;

        self
    }

    /// Add an [`AssetCollection`] to the [`AssetLoader`]
    ///
    /// The added collection will be loaded and inserted into your Bevy app as a resource.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<ImageAssets>()
    ///         .build(&mut app);
    /// #   app
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
    #[must_use]
    #[cfg(feature = "stageless")]
    pub fn with_collection<A: AssetCollection>(mut self) -> Self {
        self.loading_transition_stage.add_enter_system(
            LoadingState::LoadingAssets,
            start_loading_collection::<S, A>.exclusive_system(),
        );
        self.check_loading_assets = self
            .check_loading_assets
            .with_system(check_loading_collection::<S, A>.exclusive_system());
        self.collection_count += 1;

        self
    }

    /// Insert a map of asset keys with corresponding dynamic assets
    #[must_use]
    pub fn add_dynamic_assets(
        mut self,
        mut dynamic_assets: HashMap<String, StandardDynamicAsset>,
    ) -> Self {
        dynamic_assets.drain().for_each(|(key, value)| {
            self.dynamic_assets.insert(key, Box::new(value));
        });

        self
    }

    /// Add any [`FromWorld`] resource to be initialized after all asset collections are loaded.
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
    #[must_use]
    #[cfg(not(feature = "stageless"))]
    pub fn init_resource<A: FromWorld + Send + Sync + 'static>(mut self) -> Self {
        self.initialize_resources = self
            .initialize_resources
            .with_system(init_resource::<A>.exclusive_system());

        self
    }

    /// Add any [`FromWorld`](bevy_ecs::world::FromWorld) resource to be initialized after all asset collections are loaded.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<TextureForAtlas>()
    ///         .init_resource::<TextureAtlasFromWorld>()
    ///         .build(&mut app);
    /// #   app
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
    #[must_use]
    #[cfg(feature = "stageless")]
    pub fn init_resource<A: FromWorld + Send + Sync + 'static>(mut self) -> Self {
        self.loading_transition_stage.add_enter_system(
            LoadingState::Finalize,
            init_resource::<A>.exclusive_system(),
        );

        self
    }

    /// Register a dynamic asset collection file to be loaded and used for resolving asset keys.
    ///
    /// The file will be loaded as [`DynamicAssetCollection`].
    /// It's mapping of asset keys to dynamic assets will be used during the loading state to resolve asset keys.
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(feature = "dynamic_assets")))]
    #[cfg(feature = "dynamic_assets")]
    pub fn with_dynamic_asset_collection_file(
        mut self,
        dynamic_asset_collection_file: &str,
    ) -> Self {
        self.dynamic_asset_collections
            .push(dynamic_asset_collection_file.to_owned());

        self
    }

    /// Set all file endings loaded as dynamic asset collections.
    ///
    /// The default file ending is `.assets`
    #[must_use]
    #[cfg_attr(docsrs, doc(cfg(feature = "dynamic_assets")))]
    #[cfg(feature = "dynamic_assets")]
    pub fn set_dynamic_asset_collection_file_endings(mut self, endings: Vec<&'static str>) -> Self {
        self.dynamic_asset_collection_file_endings = endings;

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
    #[cfg(not(feature = "stageless"))]
    #[allow(unused_mut)]
    pub fn build(mut self, app: &mut App) {
        app.init_resource::<AssetLoaderConfiguration<S>>();
        app.init_resource::<LoadingStateSchedules<S>>();
        {
            let mut asset_loader_configuration = app
                .world
                .get_resource_mut::<AssetLoaderConfiguration<S>>()
                .unwrap();
            asset_loader_configuration.configuration.insert(
                self.loading_state.clone(),
                LoadingConfiguration {
                    next: self.next_state.clone(),
                    loading_collections: 0,
                },
            );
        }

        let mut loading_schedule = Schedule::default();
        let mut update = SystemStage::parallel();

        #[cfg(feature = "dynamic_assets")]
        {
            app.add_plugin(RonAssetPlugin::<StandardDynamicAssetCollection>::new(
                &self.dynamic_asset_collection_file_endings,
            ));
            update.add_system_set(
                SystemSet::on_enter(LoadingState::LoadingDynamicAssetCollections)
                    .with_system(load_dynamic_asset_collections::<S>.exclusive_system()),
            );
            update.add_system_set(
                SystemSet::on_update(LoadingState::LoadingDynamicAssetCollections)
                    .with_system(check_dynamic_asset_collections::<S>.exclusive_system()),
            );
            app.insert_resource(LoadingAssetHandles {
                handles: Default::default(),
                marker: PhantomData::<S>,
            });
            app.init_resource::<DynamicAssetCollections<S>>();
            app.world
                .get_resource_mut::<DynamicAssetCollections<S>>()
                .unwrap()
                .files
                .insert(self.loading_state.clone(), self.dynamic_asset_collections);
        }
        let mut dynamic_assets = DynamicAssets::default();
        for (key, asset) in self.dynamic_assets {
            dynamic_assets.register_asset(key, asset);
        }
        app.insert_resource(dynamic_assets);

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

    /// Finish configuring the [`AssetLoader`]
    ///
    /// Calling this function is required to set up the asset loading.
    /// ```edition2021
    /// # use bevy_asset_loader::{AssetLoader, AssetCollection};
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     let mut app = App::new();
    /// #   app
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default());
    ///     AssetLoader::new(GameState::Loading)
    ///         .continue_to_state(GameState::Menu)
    ///         .with_collection::<AudioAssets>()
    ///         .with_collection::<ImageAssets>()
    ///         .build(&mut app);
    /// #   app
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
    #[cfg(feature = "stageless")]
    #[allow(unused_mut)]
    pub fn build(mut self, app: &mut App) {
        app.init_resource::<AssetLoaderConfiguration<S>>();
        app.init_resource::<LoadingStateSchedules<S>>();
        {
            let mut asset_loader_configuration = app
                .world
                .get_resource_mut::<AssetLoaderConfiguration<S>>()
                .unwrap();
            asset_loader_configuration.configuration.insert(
                self.loading_state.clone(),
                LoadingConfiguration {
                    next: self.next_state.clone(),
                    loading_collections: 0,
                },
            );
        }

        let mut loading_schedule = Schedule::default();
        let mut update = SystemStage::parallel();

        #[cfg(feature = "dynamic_assets")]
        {
            app.add_plugin(RonAssetPlugin::<StandardDynamicAssetCollection>::new(
                &self.dynamic_asset_collection_file_endings,
            ));
            self.loading_transition_stage.add_enter_system(
                LoadingState::LoadingDynamicAssetCollections,
                load_dynamic_asset_collections::<S>.exclusive_system(),
            );
            // I think it's a bug in iyes, but you need this kind of cast to make it become a descriptor
            update.add_system(
                iyes_loopless::condition::IntoConditionalExclusiveSystem::run_in_state(
                    check_dynamic_asset_collections::<S>,
                    LoadingState::LoadingDynamicAssetCollections,
                )
                .label("iyes_loopless::condition::IntoConditionalExclusiveSystem::cast"),
            );
            app.insert_resource(LoadingAssetHandles {
                handles: Default::default(),
                marker: PhantomData::<S>,
            });
            app.init_resource::<DynamicAssetCollections<S>>();
            app.world
                .get_resource_mut::<DynamicAssetCollections<S>>()
                .unwrap()
                .files
                .insert(self.loading_state.clone(), self.dynamic_asset_collections);
        }
        let mut dynamic_assets = DynamicAssets::default();
        for (key, asset) in self.dynamic_assets {
            dynamic_assets.register_asset(key, asset);
        }
        app.insert_resource(dynamic_assets);

        update.add_system_set(
            ConditionSet::new()
                .run_in_state(LoadingState::Initialize)
                .with_system(initialize_loading_state)
                .into(),
        );
        update.add_system_set(self.check_loading_assets);
        update.add_system_set(
            ConditionSet::new()
                .run_in_state(LoadingState::Finalize)
                .with_system(finish_loading_state::<S>)
                .into(),
        );

        loading_schedule.add_stage("update", update);
        loading_schedule.add_stage_before(
            "update",
            StateTransitionStageLabel::from_type::<LoadingState>(),
            self.loading_transition_stage,
        );

        let mut loading_state_schedules = app
            .world
            .get_resource_mut::<LoadingStateSchedules<S>>()
            .unwrap();
        loading_state_schedules
            .schedules
            .insert(self.loading_state.clone(), loading_schedule);

        app.add_enter_system(self.loading_state.clone(), reset_loading_state);

        #[cfg(feature = "progress_tracking")]
        let loading_state_system =
            iyes_loopless::condition::IntoConditionalExclusiveSystem::run_in_state(
                run_loading_state::<S>,
                self.loading_state,
            )
            .at_start()
            .after(ProgressSystemLabel::Preparation);

        #[cfg(not(feature = "progress_tracking"))]
        let loading_state_system =
            iyes_loopless::condition::IntoConditionalExclusiveSystem::run_in_state(
                run_loading_state::<S>,
                self.loading_state,
            )
            .at_start();

        app.add_system_to_stage(CoreStage::Update, loading_state_system);
    }
}

/// This resource is used for handles from asset collections and loading dynamic asset collection files.
/// The generic will be the [`AssetCollection`] type for the first and the State for the second.
struct LoadingAssetHandles<T> {
    handles: Vec<HandleUntyped>,
    marker: PhantomData<T>,
}

pub struct AssetLoaderConfiguration<State: StateData> {
    configuration: HashMap<State, LoadingConfiguration<State>>,
}

impl<State: StateData> Default for AssetLoaderConfiguration<State> {
    fn default() -> Self {
        AssetLoaderConfiguration {
            configuration: HashMap::default(),
        }
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

struct LoadingConfiguration<State: StateData> {
    next: Option<State>,
    loading_collections: usize,
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
