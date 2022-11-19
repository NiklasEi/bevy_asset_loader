#[cfg(not(feature = "stageless"))]
mod dynamic_asset_systems;

#[cfg(not(feature = "stageless"))]
mod systems;

#[cfg(feature = "stageless")]
mod stageless;

use bevy::app::App;
#[cfg(feature = "stageless")]
use bevy::app::CoreStage;
use bevy::asset::{Asset, HandleUntyped};
#[cfg(not(feature = "stageless"))]
use bevy::ecs::schedule::State;
use bevy::ecs::schedule::{IntoSystemDescriptor, Schedule, StateData, SystemSet, SystemStage};
use bevy::ecs::system::{Commands, Resource};
use bevy::ecs::world::FromWorld;
use bevy::utils::HashMap;
use std::any::TypeId;
use std::marker::PhantomData;

use crate::asset_collection::AssetCollection;
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections};

#[cfg(feature = "stageless")]
use stageless::systems::{
    check_loading_collection, finish_loading_state, init_resource, initialize_loading_state,
    reset_loading_state, resume_to_finalize, run_loading_state, start_loading_collection,
};
#[cfg(not(feature = "stageless"))]
use systems::{
    check_loading_collection, finish_loading_state, init_resource, initialize_loading_state,
    reset_loading_state, resume_to_finalize, run_loading_state, start_loading_collection,
};

#[cfg(not(feature = "stageless"))]
use dynamic_asset_systems::{
    check_dynamic_asset_collections, load_dynamic_asset_collections,
    resume_to_loading_asset_collections,
};

#[cfg(feature = "stageless")]
use stageless::dynamic_asset_systems::{
    check_dynamic_asset_collections, load_dynamic_asset_collections,
    resume_to_loading_asset_collections,
};

#[cfg(feature = "standard_dynamic_assets")]
use bevy_common_assets::ron::RonAssetPlugin;

#[cfg(feature = "standard_dynamic_assets")]
use crate::standard_dynamic_asset::{StandardDynamicAsset, StandardDynamicAssetCollection};

#[cfg(feature = "progress_tracking")]
use iyes_progress::ProgressSystemLabel;

#[cfg(feature = "stageless")]
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};

#[cfg(feature = "stageless")]
use iyes_loopless::state::schedule::ScheduleLooplessStateExt;
#[cfg(feature = "stageless")]
use iyes_loopless::state::{StateTransitionStage, StateTransitionStageLabel};

use crate::dynamic_asset::{DynamicAsset, DynamicAssets};

/// A Bevy plugin to configure automatic asset loading
///
/// ```edition2021
/// # use bevy_asset_loader::prelude::*;
/// # use bevy::prelude::*;
/// # use bevy::asset::AssetPlugin;
/// # #[cfg(feature="stageless")]
/// # use iyes_loopless::prelude::*;
///
/// # #[cfg(not(feature="stageless"))]
/// fn main() {
///     App::new()
///         .add_plugins(MinimalPlugins)
/// #       .init_resource::<iyes_progress::ProgressCounter>()
///         .add_plugin(AssetPlugin::default())
///         .add_loading_state(LoadingState::new(GameState::Loading)
///             .continue_to_state(GameState::Menu)
///             .with_collection::<AudioAssets>()
///             .with_collection::<ImageAssets>()
///         )
///         .add_state(GameState::Loading)
///         .add_system_set(SystemSet::on_enter(GameState::Menu)
///             .with_system(play_audio)
///         )
/// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
///         .run();
/// }
///
/// # #[cfg(feature="stageless")]
/// # fn main() {
/// #    App::new()
/// #        .add_loopless_state(GameState::Loading)
/// #        .add_plugins(MinimalPlugins)
/// #        .init_resource::<iyes_progress::ProgressCounter>()
/// #        .add_plugin(AssetPlugin::default())
/// #        .add_loading_state(
/// #          LoadingState::new(GameState::Loading)
/// #            .continue_to_state(GameState::Menu)
/// #            .with_collection::<AudioAssets>()
/// #            .with_collection::<ImageAssets>()
/// #        )
/// #        .add_enter_system(GameState::Menu, play_audio)
/// #        .set_runner(|mut app| app.schedule.run(&mut app.world))
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
    check_loading_collections: SystemSet,
    check_loading_dynamic_collections: SystemSet,
    initialize_dependencies: SystemSet,
    start_loading_dynamic_collections: SystemSet,
    start_loading_collections: SystemSet,
    initialize_resources: SystemSet,

    dynamic_asset_collections: HashMap<TypeId, Vec<String>>,

    #[cfg(feature = "standard_dynamic_assets")]
    standard_dynamic_asset_collection_file_endings: Vec<&'static str>,
}

impl<S> LoadingState<S>
where
    S: StateData,
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
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<AudioAssets>()
    ///             .with_collection::<ImageAssets>()
    ///         )
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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
    #[cfg(not(feature = "stageless"))]
    pub fn new(load: S) -> LoadingState<S> {
        Self {
            next_state: None,
            failure_state: None,
            loading_state: load,
            dynamic_assets: HashMap::default(),
            initialize_dependencies: SystemSet::on_exit(InternalLoadingState::Initialize),
            start_loading_collections: SystemSet::on_enter(InternalLoadingState::LoadingAssets),
            start_loading_dynamic_collections: SystemSet::on_enter(
                InternalLoadingState::LoadingDynamicAssetCollections,
            ),
            check_loading_collections: SystemSet::on_update(InternalLoadingState::LoadingAssets),
            check_loading_dynamic_collections: SystemSet::on_update(
                InternalLoadingState::LoadingDynamicAssetCollections,
            ),
            initialize_resources: SystemSet::on_enter(InternalLoadingState::Finalize),
            dynamic_asset_collections: Default::default(),
            #[cfg(feature = "standard_dynamic_assets")]
            standard_dynamic_asset_collection_file_endings: vec!["assets"],
        }
    }

    /// Create a new [`LoadingState`]
    ///
    /// This function takes a [`State`](::bevy::ecs::schedule::State) during which all asset collections will
    /// be loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     App::new()
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<AudioAssets>()
    ///             .with_collection::<ImageAssets>()
    ///         )
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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
    #[cfg(feature = "stageless")]
    pub fn new(load: S) -> LoadingState<S> {
        Self {
            next_state: None,
            failure_state: None,
            loading_state: load,
            dynamic_assets: HashMap::default(),
            check_loading_collections: ConditionSet::new()
                .run_in_state(InternalLoadingState::LoadingAssets)
                .into(),
            check_loading_dynamic_collections: ConditionSet::new()
                .run_in_state(InternalLoadingState::LoadingDynamicAssetCollections)
                .into(),
            initialize_dependencies: Default::default(),
            start_loading_dynamic_collections: Default::default(),
            start_loading_collections: Default::default(),
            #[cfg(feature = "standard_dynamic_assets")]
            standard_dynamic_asset_collection_file_endings: vec!["assets"],
            dynamic_asset_collections: Default::default(),
            initialize_resources: Default::default(),
        }
    }

    /// The [`LoadingState`] will set this Bevy [`State`](::bevy::ecs::schedule::State) after all asset collections
    /// are loaded and inserted as resources.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     App::new()
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<AudioAssets>()
    ///             .with_collection::<ImageAssets>()
    ///         )
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     App::new()
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .on_failure_continue_to_state(GameState::Error)
    ///             .with_collection::<MyAssets>()
    ///         )
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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

    /// Register files to be loaded as a certain type of [`DynamicAssetCollection`]
    ///
    /// During the loading state, the given dynamic asset collections will be loaded and their
    /// content registered. This will happen before trying to resolve any dynamic assets
    /// as part of asset collections.
    ///
    /// You need to register a loader for your asset type yourself.
    /// If you want to see some code, take a look at the `custom_dynamic_assets` example.
    #[must_use]
    #[cfg(not(feature = "stageless"))]
    pub fn with_dynamic_collections<C: DynamicAssetCollection + Asset>(
        mut self,
        mut files: Vec<&str>,
    ) -> Self {
        self.dynamic_asset_collections.insert(
            TypeId::of::<C>(),
            files.drain(..).map(|file| file.to_owned()).collect(),
        );
        self.start_loading_dynamic_collections = self
            .start_loading_dynamic_collections
            .with_system(load_dynamic_asset_collections::<S, C>);
        self.check_loading_dynamic_collections = self
            .check_loading_dynamic_collections
            .with_system(check_dynamic_asset_collections::<S, C>);
        self.initialize_dependencies =
            self.initialize_dependencies
                .with_system(|mut commands: Commands| {
                    commands.init_resource::<LoadingAssetHandles<C>>();
                });

        self
    }

    /// Register files to be loaded as a certain type of [`DynamicAssetCollection`]
    ///
    /// During the loading state, the given dynamic asset collections will be loaded and their
    /// content registered. This will happen before trying to resolve any dynamic assets
    /// as part of asset collections.
    ///
    /// You need to register a loader for your asset type yourself.
    /// If you want to see some code, take a look at the `custom_dynamic_assets` example.
    #[must_use]
    #[cfg(feature = "stageless")]
    pub fn with_dynamic_collections<C: DynamicAssetCollection + Asset>(
        mut self,
        mut files: Vec<&str>,
    ) -> Self {
        self.dynamic_asset_collections.insert(
            TypeId::of::<C>(),
            files.drain(..).map(|file| file.to_owned()).collect(),
        );
        self.start_loading_dynamic_collections = self
            .start_loading_dynamic_collections
            .with_system(load_dynamic_asset_collections::<S, C>);
        self.check_loading_dynamic_collections = self
            .check_loading_dynamic_collections
            .with_system(check_dynamic_asset_collections::<S, C>);
        self.initialize_dependencies =
            self.initialize_dependencies
                .with_system(|mut commands: Commands| {
                    commands.init_resource::<LoadingAssetHandles<C>>();
                });

        self
    }

    /// Add an [`AssetCollection`] to the [`LoadingState`]
    ///
    /// The added collection will be loaded and inserted into your Bevy app as a resource.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<AudioAssets>()
    ///             .with_collection::<ImageAssets>()
    ///         )
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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
    #[cfg(not(feature = "stageless"))]
    pub fn with_collection<A: AssetCollection>(mut self) -> Self {
        self.start_loading_collections = self
            .start_loading_collections
            .with_system(start_loading_collection::<S, A>);
        self.check_loading_collections = self
            .check_loading_collections
            .with_system(check_loading_collection::<S, A>);

        self
    }

    /// Add an [`AssetCollection`] to the [`LoadingState`]
    ///
    /// The added collection will be loaded and inserted into your Bevy app as a resource.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     App::new()
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<AudioAssets>()
    ///             .with_collection::<ImageAssets>()
    ///         )
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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
    #[cfg(feature = "stageless")]
    pub fn with_collection<A: AssetCollection>(mut self) -> Self {
        self.start_loading_collections = self
            .start_loading_collections
            .with_system(start_loading_collection::<S, A>);
        self.check_loading_collections = self
            .check_loading_collections
            .with_system(check_loading_collection::<S, A>);

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

    /// Add any [`FromWorld`] resource to be initialized after all asset collections are loaded.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # fn main() {
    ///     App::new()
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<TextureForAtlas>()
    ///             .init_resource::<TextureAtlasFromWorld>()
    ///         )
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
    #[must_use]
    #[cfg(not(feature = "stageless"))]
    pub fn init_resource<A: Resource + FromWorld>(mut self) -> Self {
        self.initialize_resources = self.initialize_resources.with_system(init_resource::<A>);

        self
    }

    /// Add any [`FromWorld`](::bevy::ecs::world::FromWorld) resource to be initialized after all asset collections are loaded.
    /// ```edition2021
    /// # use bevy_asset_loader::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::asset::AssetPlugin;
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     App::new()
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<TextureForAtlas>()
    ///             .init_resource::<TextureAtlasFromWorld>()
    ///         )
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
    #[must_use]
    #[cfg(feature = "stageless")]
    pub fn init_resource<A: FromWorld + Resource>(mut self) -> Self {
        self.initialize_resources = self.initialize_resources.with_system(init_resource::<A>);

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
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<AudioAssets>()
    ///             .with_collection::<ImageAssets>()
    ///         )
    /// #       .add_state(GameState::Loading)
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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
    #[cfg(not(feature = "stageless"))]
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

        app.init_resource::<DynamicAssetCollections<S>>();
        #[cfg(feature = "standard_dynamic_assets")]
        app.add_plugin(RonAssetPlugin::<StandardDynamicAssetCollection>::new(
            &self.standard_dynamic_asset_collection_file_endings,
        ));

        let mut dynamic_asset_collections = app
            .world
            .get_resource_mut::<DynamicAssetCollections<S>>()
            .unwrap();
        let mut dynamic_collections_for_state = dynamic_asset_collections
            .files
            .remove(&self.loading_state)
            .unwrap_or_default();
        self.dynamic_asset_collections
            .drain()
            .for_each(|(id, mut files)| {
                let mut dynamic_files = dynamic_collections_for_state
                    .remove(&id)
                    .unwrap_or_default();
                dynamic_files.append(&mut files);
                dynamic_collections_for_state.insert(id, dynamic_files);
            });
        dynamic_asset_collections
            .files
            .insert(self.loading_state.clone(), dynamic_collections_for_state);

        app.insert_resource(State::new(InternalLoadingState::Initialize));
        app.init_resource::<LoadingStateSchedules<S>>();
        let mut loading_state_schedules = app
            .world
            .get_resource_mut::<LoadingStateSchedules<S>>()
            .unwrap();
        let mut loading_schedule = loading_state_schedules
            .schedules
            .remove(&self.loading_state)
            .unwrap_or_default();
        let mut update = {
            if loading_schedule
                .get_stage_mut::<SystemStage>("update")
                .is_none()
            {
                loading_schedule.add_stage("update", SystemStage::parallel());
                loading_schedule
                    .add_system_set_to_stage("update", State::<InternalLoadingState>::get_driver());
            }
            loading_schedule
                .get_stage_mut::<SystemStage>("update")
                .unwrap()
        };

        update.add_system_set(self.start_loading_dynamic_collections);
        self.check_loading_dynamic_collections = self
            .check_loading_dynamic_collections
            .with_system(resume_to_loading_asset_collections::<S>);
        update.add_system_set(self.check_loading_dynamic_collections);
        update.add_system_set(self.initialize_dependencies);

        update.add_system_set(
            SystemSet::on_update(InternalLoadingState::Initialize)
                .with_system(initialize_loading_state),
        );
        update.add_system_set(self.start_loading_collections);
        self.check_loading_collections = self
            .check_loading_collections
            .with_system(resume_to_finalize::<S>);
        update.add_system_set(self.check_loading_collections);
        update.add_system_set(self.initialize_resources);
        update.add_system_set(
            SystemSet::on_update(InternalLoadingState::Finalize)
                .with_system(finish_loading_state::<S>),
        );

        loading_state_schedules
            .schedules
            .insert(self.loading_state.clone(), loading_schedule);

        app.init_resource::<DynamicAssets>();
        let mut dynamic_assets = app.world.get_resource_mut::<DynamicAssets>().unwrap();
        for (key, asset) in self.dynamic_assets {
            dynamic_assets.register_asset(key, asset);
        }

        app.add_system_set(
            SystemSet::on_enter(self.loading_state.clone()).with_system(reset_loading_state),
        );
        #[cfg(feature = "progress_tracking")]
        let loading_state_system = run_loading_state::<S>
            .at_start()
            .after(ProgressSystemLabel::Preparation);
        #[cfg(not(feature = "progress_tracking"))]
        let loading_state_system = run_loading_state::<S>.at_start();

        app.add_system_set(
            SystemSet::on_update(self.loading_state).with_system(loading_state_system),
        );
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
    /// # use iyes_loopless::prelude::*;
    /// # fn main() {
    ///     App::new()
    /// #       .add_loopless_state(GameState::Loading)
    /// #       .add_plugins(MinimalPlugins)
    /// #       .init_resource::<iyes_progress::ProgressCounter>()
    /// #       .add_plugin(AssetPlugin::default())
    ///         .add_loading_state(
    ///           LoadingState::new(GameState::Loading)
    ///             .continue_to_state(GameState::Menu)
    ///             .with_collection::<AudioAssets>()
    ///             .with_collection::<ImageAssets>()
    ///         )
    /// #       .set_runner(|mut app| app.schedule.run(&mut app.world))
    /// #       .run();
    /// # }
    /// # #[derive(Clone, Eq, PartialEq, Debug, Hash)]
    /// # enum GameState {
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

        app.init_resource::<DynamicAssetCollections<S>>();
        #[cfg(feature = "standard_dynamic_assets")]
        app.add_plugin(RonAssetPlugin::<StandardDynamicAssetCollection>::new(
            &self.standard_dynamic_asset_collection_file_endings,
        ));

        let mut dynamic_asset_collections = app
            .world
            .get_resource_mut::<DynamicAssetCollections<S>>()
            .unwrap();
        let mut dynamic_collections_for_state = dynamic_asset_collections
            .files
            .remove(&self.loading_state)
            .unwrap_or_default();
        self.dynamic_asset_collections
            .drain()
            .for_each(|(id, mut files)| {
                let mut dynamic_files = dynamic_collections_for_state
                    .remove(&id)
                    .unwrap_or_default();
                dynamic_files.append(&mut files);
                dynamic_collections_for_state.insert(id, dynamic_files);
            });
        dynamic_asset_collections
            .files
            .insert(self.loading_state.clone(), dynamic_collections_for_state);

        app.init_resource::<DynamicAssets>();
        let mut dynamic_assets = app.world.get_resource_mut::<DynamicAssets>().unwrap();
        for (key, asset) in self.dynamic_assets {
            dynamic_assets.register_asset(key, asset);
        }

        let mut loading_state_schedules = app
            .world
            .get_resource_mut::<LoadingStateSchedules<S>>()
            .unwrap();
        let mut loading_schedule = loading_state_schedules
            .schedules
            .remove(&self.loading_state)
            .unwrap_or_default();
        let mut update = {
            if loading_schedule
                .get_stage_mut::<SystemStage>("update")
                .is_none()
            {
                loading_schedule.add_stage("update", SystemStage::parallel());
            }
            loading_schedule
                .get_stage_mut::<SystemStage>("update")
                .unwrap()
        };

        update.add_system_set(
            ConditionSet::new()
                .run_in_state(InternalLoadingState::Initialize)
                .with_system(initialize_loading_state)
                .into(),
        );

        self.check_loading_dynamic_collections = self
            .check_loading_dynamic_collections
            .with_system(resume_to_loading_asset_collections::<S>);
        update.add_system_set(self.check_loading_dynamic_collections);
        self.check_loading_collections = self
            .check_loading_collections
            .with_system(resume_to_finalize::<S>);
        update.add_system_set(self.check_loading_collections);
        update.add_system_set(
            ConditionSet::new()
                .run_in_state(InternalLoadingState::Finalize)
                .with_system(finish_loading_state::<S>)
                .into(),
        );

        if loading_schedule
            .get_stage::<StateTransitionStage<InternalLoadingState>>(
                StateTransitionStageLabel::from_type::<InternalLoadingState>(),
            )
            .is_none()
        {
            loading_schedule
                .add_loopless_state_before_stage("update", InternalLoadingState::Initialize);
        }

        loading_schedule.add_enter_system_set(
            InternalLoadingState::LoadingDynamicAssetCollections,
            self.start_loading_dynamic_collections,
        );

        loading_schedule.add_enter_system_set(
            InternalLoadingState::LoadingAssets,
            self.start_loading_collections,
        );
        loading_schedule
            .add_enter_system_set(InternalLoadingState::Finalize, self.initialize_resources);
        loading_schedule.add_exit_system_set(
            InternalLoadingState::Initialize,
            self.initialize_dependencies,
        );

        loading_state_schedules
            .schedules
            .insert(self.loading_state.clone(), loading_schedule);

        app.add_enter_system(self.loading_state.clone(), reset_loading_state);

        #[cfg(feature = "progress_tracking")]
        let loading_state_system = iyes_loopless::condition::ConditionHelpers::run_in_state(
            iyes_loopless::condition::IntoConditionalSystem::into_conditional(
                run_loading_state::<S>,
            ),
            self.loading_state,
        )
        .at_start()
        .after(ProgressSystemLabel::Preparation);

        #[cfg(not(feature = "progress_tracking"))]
        let loading_state_system = iyes_loopless::condition::ConditionHelpers::run_in_state(
            iyes_loopless::condition::IntoConditionalSystem::into_conditional(
                run_loading_state::<S>,
            ),
            self.loading_state,
        )
        .at_start();

        app.add_system_to_stage(CoreStage::Update, loading_state_system);
    }
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
pub(crate) struct AssetLoaderConfiguration<State: StateData> {
    state_configurations: HashMap<State, LoadingConfiguration<State>>,
}

impl<State: StateData> Default for AssetLoaderConfiguration<State> {
    fn default() -> Self {
        AssetLoaderConfiguration {
            state_configurations: HashMap::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum InternalLoadingState {
    /// Starting point. Here it will be decided whether or not dynamic asset collections need to be loaded.
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

struct LoadingConfiguration<State: StateData> {
    next: Option<State>,
    failure: Option<State>,
    loading_failed: bool,
    loading_collections: usize,
    loading_dynamic_collections: usize,
}

impl<State: StateData> Default for LoadingConfiguration<State> {
    fn default() -> Self {
        LoadingConfiguration {
            next: None,
            failure: None,
            loading_failed: false,
            loading_collections: 0,
            loading_dynamic_collections: 0,
        }
    }
}

/// Resource to store the schedules for loading states
#[derive(Resource)]
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

/// Extension trait for Bevy Apps to add loading states idiomatically
pub trait LoadingStateAppExt {
    /// Add a loading state to your app
    fn add_loading_state<S: StateData>(&mut self, loading_state: LoadingState<S>) -> &mut Self;
}

impl LoadingStateAppExt for App {
    fn add_loading_state<S: StateData>(&mut self, loading_state: LoadingState<S>) -> &mut Self {
        loading_state.build(self);

        self
    }
}
