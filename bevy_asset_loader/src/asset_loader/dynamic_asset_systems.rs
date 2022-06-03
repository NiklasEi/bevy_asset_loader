use crate::asset_loader::dynamic_asset::{
    DynamicAssetCollection, DynamicAssetCollections, StandardDynamicAssetCollection,
};
use crate::asset_loader::{DynamicAssets, LoadingAssetHandles, LoadingState};
use bevy::asset::{AssetServer, Assets, LoadState};
use bevy::ecs::change_detection::ResMut;
use bevy::ecs::schedule::State;
use bevy::ecs::schedule::StateData;
use bevy::ecs::system::{Res, SystemState};
use bevy::ecs::world::World;

pub(crate) fn load_dynamic_asset_collections<S: StateData>(world: &mut World) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        ResMut<DynamicAssetCollections<S>>,
        ResMut<State<LoadingState>>,
        ResMut<LoadingAssetHandles<S>>,
        Res<AssetServer>,
        Res<State<S>>,
    )> = SystemState::new(world);
    let (
        mut dynamic_asset_collections,
        mut loading_state,
        mut loading_collections,
        asset_server,
        state,
    ) = system_state.get_mut(world);

    let files = dynamic_asset_collections
        .files
        .get_mut(state.current())
        .expect("Failed to get list of dynamic asset collections for current loading state");
    if files.is_empty() {
        loading_state
            .set(LoadingState::LoadingAssets)
            .expect("Failed to set loading state");
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
        ResMut<State<LoadingState>>,
        ResMut<Assets<StandardDynamicAssetCollection>>,
        ResMut<DynamicAssets>,
    )> = SystemState::new(world);
    let (
        asset_server,
        mut loading_collections,
        mut loading_state,
        mut dynamic_asset_collections,
        mut asset_keys,
    ) = system_state.get_mut(world);

    let collections_load_state = asset_server
        .get_group_load_state(loading_collections.handles.iter().map(|handle| handle.id));
    if collections_load_state == LoadState::Loaded {
        for collection in loading_collections.handles.drain(..) {
            let collection = dynamic_asset_collections.remove(collection).unwrap();
            collection.register(&mut asset_keys);
        }
        loading_state
            .set(LoadingState::LoadingAssets)
            .expect("Failed to set loading state");
    }
}
