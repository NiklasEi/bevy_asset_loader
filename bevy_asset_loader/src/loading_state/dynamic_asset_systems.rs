use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssetCollections, DynamicAssets};
use crate::loading_state::{AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles};
use bevy::asset::{Asset, AssetServer, Assets, LoadState};
use bevy::ecs::change_detection::ResMut;
use bevy::ecs::schedule::{State, States};
use bevy::ecs::system::{Res, SystemState};
use bevy::ecs::world::World;
use bevy::prelude::NextState;
use std::any::TypeId;

#[allow(clippy::type_complexity)]
pub(crate) fn load_dynamic_asset_collections<S: States, C: DynamicAssetCollection + Asset>(
    world: &mut World,
    system_state: &mut SystemState<(
        ResMut<DynamicAssetCollections<S>>,
        ResMut<LoadingAssetHandles<C>>,
        Res<AssetServer>,
        Res<State<S>>,
        ResMut<AssetLoaderConfiguration<S>>,
    )>,
) {
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
    for file in files.remove(&TypeId::of::<C>()).unwrap_or_default() {
        loading_collections
            .handles
            .push(asset_server.load_untyped(file));
    }
    if let Some(config) = asset_loader_config.state_configurations.get_mut(&state.0) {
        config.loading_dynamic_collections.insert(TypeId::of::<C>());
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn check_dynamic_asset_collections<S: States, C: DynamicAssetCollection + Asset>(
    world: &mut World,
    system_state: &mut SystemState<(
        Res<AssetServer>,
        Option<ResMut<LoadingAssetHandles<C>>>,
        Res<State<S>>,
        Res<Assets<C>>,
        ResMut<DynamicAssets>,
        ResMut<AssetLoaderConfiguration<S>>,
    )>,
) {
    {
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
            .get_group_load_state(loading_collections.handles.iter().map(|handle| handle.id()));
        if collections_load_state != LoadState::Loaded {
            return;
        }
        for collection in loading_collections.handles.drain(..) {
            let collection = dynamic_asset_collections
                .get(&collection.typed_weak::<C>())
                .unwrap();
            collection.register(&mut asset_keys);
        }
        let config = asset_loader_config
            .state_configurations
            .get_mut(&state.0)
            .expect("No asset loader configuration for current state");
        config
            .loading_dynamic_collections
            .remove(&TypeId::of::<C>());
    }
    world.remove_resource::<LoadingAssetHandles<C>>();
}

pub(crate) fn resume_to_loading_asset_collections<S: States>(
    state: Res<State<S>>,
    mut loading_state: ResMut<NextState<InternalLoadingState>>,
    asset_loader_config: Res<AssetLoaderConfiguration<S>>,
) {
    let config = asset_loader_config
        .state_configurations
        .get(&state.0)
        .expect("No asset loader configuration for current state");
    if config.loading_dynamic_collections.is_empty() {
        loading_state.set(InternalLoadingState::LoadingAssets);
    }
}
