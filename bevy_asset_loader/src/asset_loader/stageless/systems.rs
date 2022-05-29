use bevy::asset::{AssetServer, LoadState};
use bevy::ecs::prelude::Commands;

use bevy::ecs::prelude::{FromWorld, World};
use bevy::ecs::schedule::StateData;
use bevy::ecs::system::SystemState;

use bevy::prelude::{Mut, Res, ResMut, Stage};
use std::marker::PhantomData;

#[cfg(feature = "progress_tracking")]
use iyes_progress::{Progress, ProgressCounter};

use iyes_loopless::prelude::{CurrentState, NextState};

use crate::asset_collection::AssetCollection;
use crate::asset_loader::{
    AssetLoaderConfiguration, LoadingAssetHandles, LoadingState, LoadingStateSchedules,
};

pub(crate) fn init_resource<Asset: FromWorld + Send + Sync + 'static>(world: &mut World) {
    let asset = Asset::from_world(world);
    world.insert_resource(asset);
}

pub(crate) fn start_loading_collection<S: StateData, Assets: AssetCollection>(world: &mut World) {
    #[allow(clippy::type_complexity)]
    let mut system_state: SystemState<(
        ResMut<AssetLoaderConfiguration<S>>,
        Res<CurrentState<S>>,
    )> = SystemState::new(world);
    let (mut asset_loader_configuration, state) = system_state.get_mut(world);

    let mut config = asset_loader_configuration
        .configuration
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
    if let Some((done, total, option_loading_collections)) =
        count_loaded_handles::<S, Assets>(world)
    {
        if let Some(loading_collections) = option_loading_collections {
            if loading_collections == 0 {
                world.insert_resource(NextState(LoadingState::Finalize))
            }
        }

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
) -> Option<(u32, u32, Option<usize>)> {
    let cell = world.cell();
    let loading_asset_handles = cell.get_resource::<LoadingAssetHandles<Assets>>()?;
    let total = loading_asset_handles.handles.len();

    let asset_server = cell
        .get_resource::<AssetServer>()
        .expect("Cannot get AssetServer resource");
    let done = loading_asset_handles
        .handles
        .iter()
        .map(|handle| handle.id)
        .map(|handle_id| asset_server.get_load_state(handle_id))
        .filter(|state| state == &LoadState::Loaded)
        .count();
    if done < total {
        return Some((done as u32, total as u32, None));
    }

    let mut loading_collections: Option<usize> = None;

    let state = cell
        .get_resource::<CurrentState<S>>()
        .expect("Cannot get State resource");
    let mut asset_loader_configuration = cell
        .get_resource_mut::<AssetLoaderConfiguration<S>>()
        .expect("Cannot get AssetLoaderConfiguration resource");
    if let Some(mut config) = asset_loader_configuration.configuration.get_mut(&state.0) {
        config.loading_collections -= 1;
        loading_collections = Some(config.loading_collections)
    }

    Some((done as u32, total as u32, loading_collections))
}

pub(crate) fn initialize_loading_state(mut commands: Commands) {
    #[cfg(feature = "dynamic_assets")]
    commands.insert_resource(NextState(LoadingState::LoadingDynamicAssetCollections));
    #[cfg(not(feature = "dynamic_assets"))]
    commands.insert_resource(NextState(LoadingState::LoadingAssets));
}

pub(crate) fn finish_loading_state<S: StateData>(
    mut commands: Commands,
    state: Res<CurrentState<S>>,
    asset_loader_configuration: Res<AssetLoaderConfiguration<S>>,
) {
    if let Some(config) = asset_loader_configuration.configuration.get(&state.0) {
        if let Some(next) = config.next.as_ref() {
            commands.insert_resource(NextState(next.clone()));
            return;
        }
    }

    commands.insert_resource(NextState(LoadingState::Done));
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
    commands.insert_resource(NextState(LoadingState::Initialize));
}
