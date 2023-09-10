use crate::dynamic_asset::{
    DynamicAsset, DynamicAssetCollection, DynamicAssetCollections, DynamicAssetMap, DynamicAssets,
    OneOrManyDynamicAssets,
};
use crate::loading_state::{AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles};
use bevy::asset::{AssetServer, Assets, LoadState};
use bevy::ecs::change_detection::ResMut;
use bevy::ecs::schedule::{NextState, State, States};
use bevy::ecs::system::{Res, SystemState};
use bevy::ecs::world::World;
use bevy::log::debug;
use bevy::reflect::{TypePath, TypeUuid};
use std::any::TypeId;

#[allow(clippy::type_complexity)]
pub(crate) fn load_dynamic_asset_collections<
    S: States,
    C: DynamicAsset + TypeUuid + TypePath + Clone,
>(
    world: &mut World,
    system_state: &mut SystemState<(
        Res<DynamicAssetCollections<S>>,
        Res<AssetServer>,
        Res<State<S>>,
        ResMut<AssetLoaderConfiguration<S>>,
    )>,
) {
    let (dynamic_asset_collections, asset_server, state, mut asset_loader_config) =
        system_state.get_mut(world);
    let mut loading_collections: LoadingAssetHandles<(S, C)> = LoadingAssetHandles::default();

    if let Some(files) = dynamic_asset_collections
        .get_files::<DynamicAssetMap<OneOrManyDynamicAssets<C>>>(state.get())
    {
        for file in files {
            loading_collections
                .handles
                .push(asset_server.load_untyped(file));
        }
    }
    if let Some(config) = asset_loader_config
        .state_configurations
        .get_mut(state.get())
    {
        config.loading_dynamic_collections.insert(TypeId::of::<C>());
    }
    world.insert_resource(loading_collections);
}

#[allow(clippy::type_complexity)]
pub(crate) fn check_dynamic_asset_collections<
    S: States,
    C: DynamicAsset + TypeUuid + TypePath + Clone,
>(
    world: &mut World,
    system_state: &mut SystemState<(
        Res<AssetServer>,
        Option<ResMut<LoadingAssetHandles<(S, C)>>>,
        Res<State<S>>,
        Res<Assets<DynamicAssetMap<OneOrManyDynamicAssets<C>>>>,
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
                .get(&collection.typed_weak::<DynamicAssetMap<OneOrManyDynamicAssets<C>>>())
                .unwrap();
            collection.register(&mut asset_keys);
        }
        let config = asset_loader_config
            .state_configurations
            .get_mut(state.get())
            .expect("No asset loader configuration for current state");
        config
            .loading_dynamic_collections
            .remove(&TypeId::of::<C>());
    }
    world.remove_resource::<LoadingAssetHandles<(S, C)>>();
}

pub(crate) fn resume_to_loading_asset_collections<S: States>(
    state: Res<State<S>>,
    mut loading_state: ResMut<NextState<InternalLoadingState<S>>>,
    asset_loader_config: Res<AssetLoaderConfiguration<S>>,
) {
    let config = asset_loader_config
        .state_configurations
        .get(state.get())
        .expect("No asset loader configuration for current state");
    if config.loading_dynamic_collections.is_empty() {
        debug!("No dynamic asset collection file left loading. Resuming to 'LoadingAssets'");
        loading_state.set(InternalLoadingState::LoadingAssets);
    }
}
