use crate::asset_loader::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections};
use crate::asset_loader::{DynamicAssets, LoadingAssetHandles, LoadingState};
use bevy::asset::{AssetServer, Assets, LoadState};
use bevy::ecs::change_detection::ResMut;

use bevy::ecs::schedule::StateData;
use bevy::ecs::system::{Res, SystemState};
use bevy::ecs::world::World;

use iyes_loopless::prelude::{CurrentState, NextState};

pub(crate) fn load_dynamic_asset_collections<S: StateData>(world: &mut World) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        ResMut<DynamicAssetCollections<S>>,
        ResMut<LoadingAssetHandles<S>>,
        Res<AssetServer>,
        Res<CurrentState<S>>,
    )> = SystemState::new(world);
    let (mut dynamic_asset_collections, mut loading_collections, asset_server, state) =
        system_state.get_mut(world);

    let files = dynamic_asset_collections
        .files
        .get_mut(&state.0)
        .expect("Failed to get list of dynamic asset collections for current loading state");
    if files.is_empty() {
        world.insert_resource(NextState(LoadingState::LoadingAssets));
        return;
    }
    for file in files.drain(..) {
        loading_collections
            .handles
            .push(asset_server.load_untyped(&file));
    }
}

pub(crate) fn check_dynamic_asset_collections<S: StateData>(world: &mut World) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        Res<AssetServer>,
        ResMut<LoadingAssetHandles<S>>,
        ResMut<Assets<DynamicAssetCollection>>,
        ResMut<DynamicAssets>,
    )> = SystemState::new(world);
    let (asset_server, mut loading_collections, mut dynamic_asset_collections, mut asset_keys) =
        system_state.get_mut(world);

    let collections_load_state = asset_server
        .get_group_load_state(loading_collections.handles.iter().map(|handle| handle.id));
    if collections_load_state == LoadState::Loaded {
        for collection in loading_collections.handles.drain(..) {
            let collection = dynamic_asset_collections.remove(collection).unwrap();
            asset_keys.register_dynamic_collection(collection);
        }

        world.insert_resource(NextState(LoadingState::LoadingAssets));
    }
}
