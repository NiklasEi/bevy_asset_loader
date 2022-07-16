use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections, DynamicAssets};
use crate::loading_state::{AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles};
use bevy::asset::{Asset, AssetServer, Assets, LoadState};
use bevy::ecs::change_detection::ResMut;
use bevy::ecs::schedule::StateData;
use bevy::ecs::system::{Commands, Res, SystemState};
use bevy::ecs::world::World;
use std::any::TypeId;

use iyes_loopless::prelude::{CurrentState, NextState};

pub(crate) fn load_dynamic_asset_collections<S: StateData, C: DynamicAssetCollection + Asset>(
    mut dynamic_asset_collections: ResMut<DynamicAssetCollections<S>>,
    mut loading_collections: ResMut<LoadingAssetHandles<C>>,
    asset_server: Res<AssetServer>,
    state: Res<CurrentState<S>>,
    mut asset_loader_config: ResMut<AssetLoaderConfiguration<S>>,
) {
    let files = dynamic_asset_collections
        .files
        .get_mut(&state.0)
        .expect("Failed to get list of dynamic asset collections for current loading state");
    for file in files.remove(&TypeId::of::<C>()).unwrap_or_default() {
        loading_collections
            .handles
            .push(asset_server.load_untyped(&file));
    }

    let mut config = asset_loader_config
        .configuration
        .get_mut(&state.0)
        .expect("No asset loader configuration for current state");
    config.loading_dynamic_collections += 1;
}

pub(crate) fn check_dynamic_asset_collections<S: StateData, C: DynamicAssetCollection + Asset>(
    world: &mut World,
) {
    if let Some(state) = world.get_resource::<CurrentState<InternalLoadingState>>() {
        if state.0 != InternalLoadingState::LoadingDynamicAssetCollections {
            return;
        }
    }
    {
        #[allow(clippy::type_complexity)]
        let mut system_state: SystemState<(
            Res<AssetServer>,
            Option<ResMut<LoadingAssetHandles<C>>>,
            Res<CurrentState<S>>,
            Res<Assets<C>>,
            ResMut<DynamicAssets>,
            ResMut<AssetLoaderConfiguration<S>>,
        )> = SystemState::new(world);
        let (
            asset_server,
            mut loading_collections,
            state,
            dynamic_asset_collections,
            mut asset_keys,
            mut asset_loader_config,
        ) = system_state.get_mut(world);

        if loading_collections.is_none() {
            return;
        }
        let loading_collections = loading_collections.as_mut().unwrap();
        let collections_load_state = asset_server
            .get_group_load_state(loading_collections.handles.iter().map(|handle| handle.id));
        if collections_load_state != LoadState::Loaded {
            return;
        }
        for collection in loading_collections.handles.drain(..) {
            let collection = dynamic_asset_collections
                .get(&collection.typed_weak::<C>())
                .unwrap();
            collection.register(&mut asset_keys);
        }
        let mut config = asset_loader_config
            .configuration
            .get_mut(&state.0)
            .expect("No asset loader configuration for current state");
        config.loading_dynamic_collections -= 1;
    }
    world.remove_resource::<LoadingAssetHandles<C>>();
}

pub(crate) fn resume_to_loading_asset_collections<S: StateData>(
    mut commands: Commands,
    state: Res<CurrentState<S>>,
    internal_state: Res<CurrentState<InternalLoadingState>>,
    asset_loader_config: Res<AssetLoaderConfiguration<S>>,
) {
    if internal_state.0 != InternalLoadingState::LoadingDynamicAssetCollections {
        return;
    }
    let config = asset_loader_config
        .configuration
        .get(&state.0)
        .expect("No asset loader configuration for current state");
    if config.loading_dynamic_collections == 0 {
        commands.insert_resource(NextState(InternalLoadingState::LoadingAssets));
    }
}
