use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections, DynamicAssets};
use crate::loading_state::{AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles};
use bevy::asset::{Asset, AssetServer, Assets, LoadState};
use bevy::ecs::change_detection::ResMut;
use bevy::ecs::schedule::State;
use bevy::ecs::schedule::StateData;
use bevy::ecs::system::{Res, SystemState};
use bevy::ecs::world::World;
use std::any::TypeId;

pub(crate) fn load_dynamic_asset_collections<S: StateData, C: DynamicAssetCollection + Asset>(
    world: &mut World,
) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        ResMut<DynamicAssetCollections<S>>,
        ResMut<LoadingAssetHandles<C>>,
        Res<AssetServer>,
        Res<State<S>>,
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
        .get_mut(state.current())
        .expect("Failed to get list of dynamic asset collections for current loading state");
    for file in files.remove(&TypeId::of::<C>()).unwrap_or_default() {
        loading_collections
            .handles
            .push(asset_server.load_untyped(&file));
    }
    if let Some(mut config) = asset_loader_config.configuration.get_mut(state.current()) {
        config.loading_dynamic_collections += 1;
    }
}

pub(crate) fn check_dynamic_asset_collections<S: StateData, C: DynamicAssetCollection + Asset>(
    world: &mut World,
) {
    {
        #[allow(clippy::type_complexity)]
        let mut system_state: SystemState<(
            Res<AssetServer>,
            Option<ResMut<LoadingAssetHandles<C>>>,
            Res<State<S>>,
            ResMut<State<InternalLoadingState>>,
            Res<Assets<C>>,
            ResMut<DynamicAssets>,
            ResMut<AssetLoaderConfiguration<S>>,
        )> = SystemState::new(world);
        let (
            asset_server,
            mut loading_collections,
            state,
            mut loading_state,
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
            let collection = dynamic_asset_collections.get(collection).unwrap();
            collection.register(&mut asset_keys);
        }
        if let Some(mut config) = asset_loader_config.configuration.get_mut(state.current()) {
            config.loading_dynamic_collections -= 1;
            if config.loading_dynamic_collections == 0 {
                loading_state
                    .set(InternalLoadingState::LoadingAssets)
                    .expect("Failed to set loading State");
            }
        }
    }
    world.remove_resource::<LoadingAssetHandles<C>>();
}
