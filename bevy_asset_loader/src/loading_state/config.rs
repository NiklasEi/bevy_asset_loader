use crate::asset_collection::AssetCollection;
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections};
use crate::loading_state::dynamic_asset_systems::{
    check_dynamic_asset_collections, load_dynamic_asset_collections,
};
use crate::loading_state::systems::{
    check_loading_collection, finally_init_resource, start_loading_collection,
};
use crate::loading_state::{
    InternalLoadingState, InternalLoadingStateSet, LoadingStateSchedule,
    OnEnterInternalLoadingState,
};
use bevy::app::App;
use bevy::asset::Asset;
use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::{default, FromWorld, IntoSystemConfigs, Resource};
use bevy::state::state::FreelyMutableState;
use bevy::utils::HashMap;
use std::any::TypeId;

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
    fn register_dynamic_asset_collection<C: DynamicAssetCollection + Asset>(self) -> Self;

    /// Add a file containing dynamic assets to the loading state. Keys contained in the file, will
    /// be available for asset collections.
    ///
    /// See the `dynamic_asset` example
    #[must_use = "The configuration will only be applied when passed to App::configure_loading_state"]
    fn with_dynamic_assets_file<C: DynamicAssetCollection + Asset>(self, file: &str) -> Self;
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

    on_enter_loading_assets: Vec<SystemConfigs>,
    on_enter_loading_dynamic_asset_collections: Vec<SystemConfigs>,
    on_update: Vec<SystemConfigs>,
    on_enter_finalize: Vec<SystemConfigs>,

    dynamic_assets: HashMap<TypeId, Vec<String>>,
}

impl<S: FreelyMutableState> LoadingStateConfig<S> {
    /// Create a new configuration for the given loading state
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

    pub(crate) fn with_dynamic_assets_type_id(&mut self, file: &str, type_id: TypeId) {
        let mut dynamic_files = self.dynamic_assets.remove(&type_id).unwrap_or_default();
        dynamic_files.push(file.to_owned());
        self.dynamic_assets.insert(type_id, dynamic_files);
    }

    pub(crate) fn build(mut self, app: &mut App) {
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
        for config in self.on_enter_loading_dynamic_asset_collections.drain(..) {
            app.add_systems(
                OnEnterInternalLoadingState(
                    self.state.clone(),
                    InternalLoadingState::LoadingDynamicAssetCollections,
                ),
                config,
            );
        }
        let mut dynamic_assets = app
            .world_mut()
            .get_resource_mut::<DynamicAssetCollections<S>>()
            .unwrap_or_else(|| {
                panic!("Failed to get the DynamicAssetCollections resource for the loading state. Are you trying to configure a loading state before it was added to the bevy App?")
            });
        for (id, files) in self.dynamic_assets.drain() {
            dynamic_assets.register_files_by_type_id(self.state.clone(), files, id);
        }
    }
}

impl<S: FreelyMutableState> ConfigureLoadingState for LoadingStateConfig<S> {
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

    fn finally_init_resource<R: Resource + FromWorld>(mut self) -> Self {
        self.on_enter_finalize
            .push(finally_init_resource::<R>.into_configs());

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
