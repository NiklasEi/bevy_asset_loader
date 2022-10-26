use bevy::asset::{AssetServer, LoadState};
use bevy::ecs::schedule::{State, StateData};
use bevy::ecs::system::{Resource, SystemState};
use bevy::ecs::world::{FromWorld, World, WorldCell};
use bevy::log::warn;
use bevy::prelude::{Mut, Res, ResMut, Stage};
use std::marker::PhantomData;

#[cfg(feature = "progress_tracking")]
use iyes_progress::{Progress, ProgressCounter};

use crate::asset_collection::AssetCollection;
use crate::loading_state::{
    AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles, LoadingStateSchedules,
};

pub(crate) fn init_resource<Asset: Resource + FromWorld + Send + Sync + 'static>(
    world: &mut World,
) {
    let asset = Asset::from_world(world);
    world.insert_resource(asset);
}

pub(crate) fn start_loading_collection<S: StateData, Assets: AssetCollection>(world: &mut World) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(ResMut<AssetLoaderConfiguration<S>>, Res<State<S>>)> =
        SystemState::new(world);
    let (mut asset_loader_configuration, state) = system_state.get_mut(world);

    let mut config = asset_loader_configuration
        .state_configurations
        .get_mut(state.current())
        .unwrap_or_else(|| {
            panic!(
                "Could not find a loading configuration for state {:?}",
                state.current()
            )
        });
    config.loading_collections += 1;
    let handles = LoadingAssetHandles {
        handles: Assets::load(world),
        marker: PhantomData::<Assets>,
    };
    world.insert_resource(handles);
}

pub(crate) fn check_loading_collection<S: StateData, Assets: AssetCollection>(world: &mut World) {
    if let Some((done, total)) = count_loaded_handles::<S, Assets>(world.cell()) {
        if total == done {
            let asset_collection = Assets::create(world);
            world.insert_resource(asset_collection);
            world.remove_resource::<LoadingAssetHandles<Assets>>();

            #[cfg(feature = "progress_tracking")]
            world
                .resource_mut::<ProgressCounter>()
                .persist_progress(Progress { done, total });
        } else {
            #[cfg(feature = "progress_tracking")]
            world
                .resource::<ProgressCounter>()
                .manually_track(Progress { done, total });
        }
    }
}

fn count_loaded_handles<S: StateData, Assets: AssetCollection>(
    cell: WorldCell,
) -> Option<(u32, u32)> {
    let loading_asset_handles = cell.get_resource::<LoadingAssetHandles<Assets>>()?;
    let total = loading_asset_handles.handles.len();

    let asset_server = cell
        .get_resource::<AssetServer>()
        .expect("Cannot get AssetServer resource");
    let failure = loading_asset_handles
        .handles
        .iter()
        .map(|handle| handle.id)
        .find(|handle_id| asset_server.get_load_state(*handle_id) == LoadState::Failed)
        .is_some();
    let done = loading_asset_handles
        .handles
        .iter()
        .map(|handle| handle.id)
        .map(|handle_id| asset_server.get_load_state(handle_id))
        .filter(|state| state == &LoadState::Loaded)
        .count();
    if done < total && !failure {
        return Some((done as u32, total as u32));
    }

    let state = cell
        .get_resource::<State<S>>()
        .expect("Cannot get State resource");
    let mut asset_loader_configuration = cell
        .get_resource_mut::<AssetLoaderConfiguration<S>>()
        .expect("Cannot get AssetLoaderConfiguration resource");
    if let Some(mut config) = asset_loader_configuration
        .state_configurations
        .get_mut(state.current())
    {
        if failure {
            config.loading_failed = true;
        } else {
            config.loading_collections -= 1;
        }
    } else {
        warn!("Failed to read loading state configuration in count_loaded_handles")
    }

    Some((done as u32, total as u32))
}

pub(crate) fn resume_to_finalize<S: StateData>(
    loader_configuration: Res<AssetLoaderConfiguration<S>>,
    mut internal_state: ResMut<State<InternalLoadingState>>,
    mut user_state: ResMut<State<S>>,
) {
    if let Some(configuration) = loader_configuration
        .state_configurations
        .get(user_state.current())
    {
        if configuration.loading_collections == 0 {
            internal_state
                .overwrite_set(InternalLoadingState::Finalize)
                .expect("Failed to set internal state to Finalize");
        }
        if configuration.loading_failed && configuration.failure.is_some() {
            let failure = configuration.failure.clone().unwrap();
            user_state
                .overwrite_set(failure)
                .expect("Failed to set loading state to failed");
        }
    } else {
        warn!("Failed to read loading state configuration in resume_to_finalize")
    }
}

pub(crate) fn initialize_loading_state(mut loading_state: ResMut<State<InternalLoadingState>>) {
    loading_state
        .overwrite_set(InternalLoadingState::LoadingDynamicAssetCollections)
        .expect("Failed to set LoadingState");
}

pub(crate) fn finish_loading_state<S: StateData>(
    mut state: ResMut<State<S>>,
    mut loading_state: ResMut<State<InternalLoadingState>>,
    asset_loader_configuration: Res<AssetLoaderConfiguration<S>>,
) {
    if let Some(config) = asset_loader_configuration
        .state_configurations
        .get(state.current())
    {
        if let Some(next) = config.next.as_ref() {
            // Overwrite to support multiple loading states configured in one Bevy State
            state
                .overwrite_set(next.clone())
                .expect("Failed to set next State");
            return;
        }
    }

    loading_state
        .set(InternalLoadingState::Done)
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

pub(crate) fn reset_loading_state(mut state: ResMut<State<InternalLoadingState>>) {
    // we can ignore the error, because it means we are already in the correct state
    let _ = state.overwrite_set(InternalLoadingState::Initialize);
}
