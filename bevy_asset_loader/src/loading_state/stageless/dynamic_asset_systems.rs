use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections, DynamicAssets};
use crate::loading_state::{AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles};
use crate::standard_dynamic_asset::StandardDynamicAssetCollection;
use bevy::asset::{AssetServer, Assets, LoadState};
use bevy::ecs::change_detection::ResMut;

use bevy::ecs::schedule::{State, StateData};
use bevy::ecs::system::{Res, SystemState};
use bevy::ecs::world::World;

use iyes_loopless::prelude::{CurrentState, NextState};

pub(crate) fn load_dynamic_asset_collections<S: StateData, C: DynamicAssetCollection + Asset>(
    world: &mut World,
) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        ResMut<DynamicAssetCollections<S>>,
        ResMut<LoadingAssetHandles<C>>,
        Res<AssetServer>,
        Res<CurrentState<S>>,
        ResMut<AssetLoaderConfiguration<S>>,
    )> = SystemState::new(world);
    let (
        mut dynamic_asset_collections,
        mut loading_collections,
        asset_server,
        state,
        mut asset_loader_config,
    ) = system_state.get_mut(world);

    let files = dynamic_asset_collections
        .files
        .get_mut(&state.0)
        .expect("Failed to get list of dynamic asset collections for current loading state");
    for file in files.drain(..) {
        loading_collections
            .handles
            .push(asset_server.load_untyped(&file));
    }
    if let Some(mut config) = asset_loader_config.configuration.get_mut(&state.0) {
        config.loading_dynamic_collections += 1;
    }
}

pub(crate) fn check_dynamic_asset_collections<S: StateData, C: DynamicAssetCollection + Asset>(
    world: &mut World,
) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        Res<AssetServer>,
        ResMut<LoadingAssetHandles<C>>,
        Res<CurrentState<S>>,
        ResMut<Assets<C>>,
        ResMut<DynamicAssets>,
        ResMut<AssetLoaderConfiguration<S>>,
    )> = SystemState::new(world);
    let (
        asset_server,
        mut loading_collections,
        state,
        mut dynamic_asset_collections,
        mut asset_keys,
        mut asset_loader_config,
    ) = system_state.get_mut(world);

    let collections_load_state = asset_server
        .get_group_load_state(loading_collections.handles.iter().map(|handle| handle.id));
    if collections_load_state == LoadState::Loaded {
        for collection in loading_collections.handles.drain(..) {
            let collection = dynamic_asset_collections.get(collection).unwrap();
            collection.register(&mut asset_keys);
        }

        if let Some(mut config) = asset_loader_config.configuration.get_mut(&state.0) {
            config.loading_dynamic_collections -= 1;
            if config.loading_dynamic_collections == 0 {
                world.insert_resource(NextState(InternalLoadingState::LoadingAssets));
            }
        }
    }
}
