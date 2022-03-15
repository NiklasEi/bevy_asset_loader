use bevy::asset::{AssetServer, LoadState};
use bevy::ecs::prelude::{FromWorld, State, World};
use bevy::ecs::schedule::StateData;
use bevy::prelude::{Mut, Res, ResMut, Stage};
use std::marker::PhantomData;

use crate::asset_collection::AssetCollection;
use crate::asset_loader::{
    AssetLoaderConfiguration, LoadingAssetHandles, LoadingState, LoadingStateSchedules,
};

pub(crate) fn init_resource<Asset: FromWorld + Send + Sync + 'static>(world: &mut World) {
    let asset = Asset::from_world(world);
    world.insert_resource(asset);
}

pub(crate) fn start_loading_collection<S: StateData, Assets: AssetCollection>(world: &mut World) {
    {
        let cell = world.cell();
        let mut asset_loader_configuration = cell
            .get_resource_mut::<AssetLoaderConfiguration<S>>()
            .expect("Cannot get AssetLoaderConfiguration");
        let state = cell.get_resource::<State<S>>().expect("Cannot get state");
        let mut config = asset_loader_configuration
            .configuration
            .get_mut(state.current())
            .unwrap_or_else(|| {
                panic!(
                    "Could not find a loading configuration for state {:?}",
                    state.current()
                )
            });
        config.count += 1;
    }
    let handles = LoadingAssetHandles {
        handles: Assets::load(world),
        marker: PhantomData::<Assets>,
    };
    world.insert_resource(handles);
}

pub(crate) fn check_loading_collection<S: StateData, Assets: AssetCollection>(world: &mut World) {
    {
        let cell = world.cell();

        let loading_asset_handles = cell.get_resource::<LoadingAssetHandles<Assets>>();
        if loading_asset_handles.is_none() {
            return;
        }
        let loading_asset_handles = loading_asset_handles.unwrap();

        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Cannot get AssetServer resource");
        let load_state = asset_server
            .get_group_load_state(loading_asset_handles.handles.iter().map(|handle| handle.id));
        if load_state != LoadState::Loaded {
            return;
        }

        let state = cell
            .get_resource::<State<S>>()
            .expect("Cannot get State resource");
        let mut loading_state = cell
            .get_resource_mut::<State<LoadingState>>()
            .expect("Cannot get LoadingStatePhase");
        let mut asset_loader_configuration = cell
            .get_resource_mut::<AssetLoaderConfiguration<S>>()
            .expect("Cannot get AssetLoaderConfiguration resource");
        if let Some(mut config) = asset_loader_configuration
            .configuration
            .get_mut(state.current())
        {
            config.count -= 1;
            if config.count == 0 {
                loading_state
                    .set(LoadingState::Finalize)
                    .expect("Failed to set loading State");
            }
        }
    }
    let asset_collection = Assets::create(world);
    world.insert_resource(asset_collection);
    world.remove_resource::<LoadingAssetHandles<Assets>>();
}

pub(crate) fn initialize_loading_state(mut loading_state: ResMut<State<LoadingState>>) {
    #[cfg(feature = "dynamic_assets")]
    loading_state
        .set(LoadingState::LoadingDynamicAssetCollections)
        .expect("Failed to set LoadingState");
    #[cfg(not(feature = "dynamic_assets"))]
    loading_state
        .set(LoadingState::LoadingAssets)
        .expect("Failed to set LoadingState");
}

pub(crate) fn finish_loading_state<S: StateData>(
    mut state: ResMut<State<S>>,
    mut loading_state: ResMut<State<LoadingState>>,
    asset_loader_configuration: Res<AssetLoaderConfiguration<S>>,
) {
    if let Some(config) = asset_loader_configuration
        .configuration
        .get(state.current())
    {
        if let Some(next) = config.next.as_ref() {
            state.set(next.clone()).expect("Failed to set next State");
            return;
        }
    }

    loading_state
        .set(LoadingState::Done)
        .expect("Failed to set LoadingState");
}

pub(crate) fn run_loading_state<S: StateData>(world: &mut World) {
    world.resource_scope(
        |world, mut loading_state_config: Mut<LoadingStateSchedules<S>>| {
            if let Some(schedule) = loading_state_config
                .schedules
                .get_mut(world.get_resource::<State<S>>().unwrap().current())
            {
                schedule.run(world);
            }
        },
    );
}

pub(crate) fn reset_loading_state(mut state: ResMut<State<LoadingState>>) {
    // we can ignore the error, because it means we are already in the correct state
    let _ = state.overwrite_set(LoadingState::Initialize);
}
