use bevy::asset::{AssetServer, LoadState};
use bevy::ecs::prelude::Commands;

use bevy::ecs::change_detection::{Mut, ResMut};
use bevy::ecs::prelude::{FromWorld, World};
use bevy::ecs::schedule::{Stage, StateData};
use bevy::ecs::system::{Res, SystemState};

use bevy::log::warn;
use bevy::prelude::Resource;
use std::marker::PhantomData;

#[cfg(feature = "progress_tracking")]
use iyes_progress::{Progress, ProgressCounter};

use iyes_loopless::prelude::{CurrentState, NextState};

use crate::asset_collection::AssetCollection;
use crate::loading_state::{
    AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles, LoadingStateSchedules,
};

pub(crate) fn init_resource<Asset: FromWorld + Resource>(world: &mut World) {
    let asset = Asset::from_world(world);
    world.insert_resource(asset);
}

#[allow(clippy::type_complexity)]
pub(crate) fn start_loading_collection<S: StateData, Assets: AssetCollection>(
    world: &mut World,
    system_state: &mut SystemState<(ResMut<AssetLoaderConfiguration<S>>, Res<CurrentState<S>>)>,
) {
    let (mut asset_loader_configuration, state) = system_state.get_mut(world);

    let mut config = asset_loader_configuration
        .state_configurations
        .get_mut(&state.0)
        .unwrap_or_else(|| {
            panic!(
                "Could not find a loading configuration for state {:?}",
                state
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
    if let Some(state) = world.get_resource::<CurrentState<InternalLoadingState>>() {
        if state.0 != InternalLoadingState::LoadingAssets {
            return;
        }
    }
    if let Some((done, total)) = count_loaded_handles::<S, Assets>(world) {
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
    world: &mut World,
) -> Option<(u32, u32)> {
    let cell = world.cell();
    let loading_asset_handles = cell.get_resource::<LoadingAssetHandles<Assets>>()?;
    let total = loading_asset_handles.handles.len();

    let asset_server = cell
        .get_resource::<AssetServer>()
        .expect("Cannot get AssetServer resource");
    let failure = loading_asset_handles
        .handles
        .iter()
        .map(|handle| handle.id)
        .any(|handle_id| asset_server.get_load_state(handle_id) == LoadState::Failed);
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
        .get_resource::<CurrentState<S>>()
        .expect("Cannot get State resource");
    let mut asset_loader_configuration = cell
        .get_resource_mut::<AssetLoaderConfiguration<S>>()
        .expect("Cannot get AssetLoaderConfiguration resource");
    if let Some(mut config) = asset_loader_configuration
        .state_configurations
        .get_mut(&state.0)
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
    mut commands: Commands,
    loader_configuration: Res<AssetLoaderConfiguration<S>>,
    state: Res<CurrentState<S>>,
    internal_state: Res<CurrentState<InternalLoadingState>>,
) {
    if internal_state.0 != InternalLoadingState::LoadingAssets {
        return;
    }
    if let Some(configuration) = loader_configuration.state_configurations.get(&state.0) {
        if configuration.loading_collections == 0 {
            commands.insert_resource(NextState(InternalLoadingState::Finalize))
        }
        if configuration.loading_failed && configuration.failure.is_some() {
            let failure = configuration.failure.clone().unwrap();
            commands.insert_resource(NextState(failure))
        }
    }
}

pub(crate) fn initialize_loading_state(
    mut commands: Commands,
    internal_state: Res<CurrentState<InternalLoadingState>>,
) {
    if internal_state.0 != InternalLoadingState::Initialize {
        return;
    }
    commands.insert_resource(NextState(
        InternalLoadingState::LoadingDynamicAssetCollections,
    ));
}

pub(crate) fn finish_loading_state<S: StateData>(
    mut commands: Commands,
    state: Res<CurrentState<S>>,
    asset_loader_configuration: Res<AssetLoaderConfiguration<S>>,
) {
    if let Some(config) = asset_loader_configuration
        .state_configurations
        .get(&state.0)
    {
        if let Some(next) = config.next.as_ref() {
            commands.insert_resource(NextState(next.clone()));
            return;
        }
    }

    commands.insert_resource(NextState(InternalLoadingState::Done));
}

pub(crate) fn run_loading_state<S: StateData>(world: &mut World) {
    world.resource_scope(
        |world, mut loading_state_config: Mut<LoadingStateSchedules<S>>| {
            if let Some(schedule) = loading_state_config
                .schedules
                .get_mut(&world.get_resource::<CurrentState<S>>().unwrap().0)
            {
                schedule.run(world);
            }
        },
    );
}

pub(crate) fn reset_loading_state(mut commands: Commands) {
    commands.insert_resource(CurrentState(InternalLoadingState::Initialize));
}
