use bevy::asset::{AssetServer, LoadState};
use bevy::ecs::schedule::{State, States};
use bevy::ecs::system::SystemState;
use bevy::ecs::world::{FromWorld, World, WorldCell};
use bevy::log::{debug, info, trace, warn};
use bevy::prelude::{NextState, Res, ResMut, Resource, Schedules};
use std::any::{type_name, TypeId};
use std::marker::PhantomData;

#[cfg(feature = "progress_tracking")]
use iyes_progress::{HiddenProgress, Progress, ProgressCounter};

use crate::asset_collection::AssetCollection;
use crate::loading_state::{
    AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles, LoadingStateSchedule,
    OnEnterInternalLoadingState,
};

pub(crate) fn init_resource<Asset: Resource + FromWorld>(world: &mut World) {
    let asset = Asset::from_world(world);
    world.insert_resource(asset);
}

#[allow(clippy::type_complexity)]
pub(crate) fn start_loading_collection<S: States, Assets: AssetCollection>(
    world: &mut World,
    system_state: &mut SystemState<(ResMut<AssetLoaderConfiguration<S>>, Res<State<S>>)>,
) {
    debug!(
        "Starting to load collection for type id {:?}",
        TypeId::of::<Assets>()
    );
    let (mut asset_loader_configuration, state) = system_state.get_mut(world);

    let config = asset_loader_configuration
        .state_configurations
        .get_mut(state.get())
        .unwrap_or_else(|| {
            panic!(
                "Could not find a loading configuration for state {:?}",
                &state
            )
        });
    if !config.loading_collections.insert(TypeId::of::<Assets>()) {
        warn!(
            "The asset collection '{}' was added multiple times to the loading state '{:?}'",
            type_name::<Assets>(),
            state.get()
        );
    }
    let handles = LoadingAssetHandles {
        handles: Assets::load(world),
        marker: PhantomData::<Assets>,
    };
    world.insert_resource(handles);
}

pub(crate) fn check_loading_collection<S: States, Assets: AssetCollection>(world: &mut World) {
    debug!(
        "Check loading of collection for type id {:?}",
        TypeId::of::<Assets>()
    );
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

fn count_loaded_handles<S: States, Assets: AssetCollection>(cell: WorldCell) -> Option<(u32, u32)> {
    let loading_asset_handles = cell.get_resource::<LoadingAssetHandles<Assets>>()?;
    let total = loading_asset_handles.handles.len();

    let asset_server = cell
        .get_resource::<AssetServer>()
        .expect("Cannot get AssetServer resource");
    let failure = loading_asset_handles
        .handles
        .iter()
        .any(|handle| asset_server.get_load_state(handle.id()) == Some(LoadState::Failed));
    let done = loading_asset_handles
        .handles
        .iter()
        .map(|handle| asset_server.get_load_state(handle.id()))
        .filter(|state| state == &Some(LoadState::Loaded))
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
    if let Some(config) = asset_loader_configuration
        .state_configurations
        .get_mut(state.get())
    {
        if failure {
            config.loading_failed = true;
        } else {
            config.loading_collections.remove(&TypeId::of::<Assets>());
        }
    } else {
        warn!("Failed to read loading state configuration in count_loaded_handles");
    }

    Some((done as u32, total as u32))
}

pub(crate) fn resume_to_finalize<S: States>(
    loader_configuration: Res<AssetLoaderConfiguration<S>>,
    mut internal_state: ResMut<NextState<InternalLoadingState<S>>>,
    user_state: Res<State<S>>,
    mut next_user_state: ResMut<NextState<S>>,
) {
    if let Some(configuration) = loader_configuration
        .state_configurations
        .get(user_state.get())
    {
        if configuration.loading_collections.is_empty() {
            internal_state.set(InternalLoadingState::Finalize);
        }
        if configuration.loading_failed && configuration.failure.is_some() {
            let failure = configuration.failure.clone().unwrap();
            next_user_state.set(failure);
        }
    } else {
        warn!("Failed to read loading state configuration in resume_to_finalize");
    }
}

pub(crate) fn initialize_loading_state<S: States>(
    mut loading_state: ResMut<NextState<InternalLoadingState<S>>>,
    #[cfg(feature = "progress_tracking")] mut progress_counter: ResMut<ProgressCounter>,
) {
    #[cfg(feature = "progress_tracking")]
    progress_counter.persist_progress_hidden(HiddenProgress(Progress { total: 1, done: 0 }));
    loading_state.set(InternalLoadingState::LoadingDynamicAssetCollections);
}

pub(crate) fn finish_loading_state<S: States>(
    state: Res<State<S>>,
    mut next_state: ResMut<NextState<S>>,
    #[cfg(feature = "progress_tracking")] mut progress_counter: ResMut<ProgressCounter>,
    mut loading_state: ResMut<NextState<InternalLoadingState<S>>>,
    asset_loader_configuration: Res<AssetLoaderConfiguration<S>>,
) {
    #[cfg(feature = "progress_tracking")]
    progress_counter.persist_progress_hidden(HiddenProgress(Progress { total: 0, done: 1 }));
    info!(
        "Loading state '{}::{:?}' is done",
        type_name::<S>(),
        state.get()
    );
    if let Some(config) = asset_loader_configuration
        .state_configurations
        .get(state.get())
    {
        if let Some(next) = config.next.as_ref() {
            next_state.set(next.clone());
            return;
        }
    }

    loading_state.set(InternalLoadingState::Done(PhantomData));
}

pub(crate) fn reset_loading_state<S: States>(world: &mut World) {
    world.remove_resource::<State<InternalLoadingState<S>>>();
    world.init_resource::<State<InternalLoadingState<S>>>();
}

pub(crate) fn run_loading_state<S: States>(world: &mut World) {
    let state = world.resource::<State<S>>().get().clone();
    world.run_schedule(LoadingStateSchedule(state));
}

pub fn apply_internal_state_transition<S: States>(world: &mut World) {
    let state = world.resource::<State<S>>().get().clone();
    if world
        .resource::<NextState<InternalLoadingState<S>>>()
        .0
        .is_some()
    {
        let entered_state = world
            .resource_mut::<NextState<InternalLoadingState<S>>>()
            .0
            .take()
            .unwrap();

        let exited_state = world.remove_resource::<State<InternalLoadingState<S>>>();
        world.insert_resource(State::new(entered_state.clone()));
        trace!(
            "Switching internal state of loading state from {exited_state:?} to {entered_state:?}"
        );
        if world
            .resource::<Schedules>()
            .contains(OnEnterInternalLoadingState(
                state.clone(),
                entered_state.clone(),
            ))
        {
            world.run_schedule(OnEnterInternalLoadingState(state, entered_state));
        }
    }
}
