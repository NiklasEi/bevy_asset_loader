use crate::asset_collection::AssetCollection;
#[cfg(feature = "progress_tracking")]
use crate::loading_state::{AssetCollectionsProgressId, LoadingStateProgressId};
use crate::loading_state::{
    AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles, LoadingStateSchedule,
    OnEnterInternalLoadingState,
};
use bevy_asset::AssetServer;
use bevy_ecs::schedule::Schedules;
use bevy_ecs::system::{Res, ResMut, Resource, SystemState};
use bevy_ecs::world::{FromWorld, World};
use bevy_log::{debug, info, trace, warn};
use bevy_state::state::{FreelyMutableState, NextState, State};
#[cfg(feature = "progress_tracking")]
use iyes_progress::{ProgressEntryId, ProgressTracker};
use std::any::{TypeId, type_name};
use std::marker::PhantomData;

pub(crate) fn finally_init_resource<Asset: Resource + FromWorld>(world: &mut World) {
    world.init_resource::<Asset>();
}

#[allow(clippy::type_complexity)]
pub(crate) fn start_loading_collection<S: FreelyMutableState, Assets: AssetCollection>(
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

    #[cfg(feature = "progress_tracking")]
    world.insert_resource(AssetCollectionsProgressId::<S, Assets>::new(
        ProgressEntryId::new(),
    ));
}

#[allow(clippy::type_complexity)]
pub(crate) fn check_loading_collection<S: FreelyMutableState, Assets: AssetCollection>(
    world: &mut World,
    system_state: &mut SystemState<(
        Option<Res<LoadingAssetHandles<Assets>>>,
        Res<State<S>>,
        Res<AssetServer>,
        ResMut<AssetLoaderConfiguration<S>>,
    )>,
) {
    debug!(
        "Check loading of collection for type id {:?}",
        TypeId::of::<Assets>()
    );
    let (loading_asset_handles, state, asset_server, mut asset_loader_configuration) =
        system_state.get_mut(world);

    if let Some(loading_asset_handles) = loading_asset_handles {
        let (done, total) = count_loaded_handles::<S, Assets>(
            &loading_asset_handles,
            &state,
            &asset_server,
            &mut asset_loader_configuration,
        );
        #[cfg(feature = "progress_tracking")]
        {
            let entry_id = world.resource::<AssetCollectionsProgressId<S, Assets>>().id;
            if let Some(tracker) = world.get_resource::<ProgressTracker<S>>() {
                tracker.set_progress(entry_id, done, total);
            }
        }
        if total == done {
            let asset_collection = Assets::create(world);
            world.insert_resource(asset_collection);
            world.remove_resource::<LoadingAssetHandles<Assets>>();
        }
    }
}

fn count_loaded_handles<S: FreelyMutableState, Assets: AssetCollection>(
    loading_asset_handles: &LoadingAssetHandles<Assets>,
    state: &State<S>,
    asset_server: &AssetServer,
    asset_loader_configuration: &mut AssetLoaderConfiguration<S>,
) -> (u32, u32) {
    let total = loading_asset_handles.handles.len();

    let failure = loading_asset_handles.handles.iter().any(|handle| {
        asset_server
            .get_recursive_dependency_load_state(handle.id())
            .map(|state| state.is_failed())
            .unwrap_or(false)
    });
    let done = loading_asset_handles
        .handles
        .iter()
        .filter(|handle| asset_server.is_loaded_with_dependencies(handle.id()))
        .count();
    if done < total && !failure {
        return (done as u32, total as u32);
    }

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

    (done as u32, total as u32)
}

pub(crate) fn resume_to_finalize<S: FreelyMutableState>(
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

pub(crate) fn initialize_loading_state<S: FreelyMutableState>(
    mut loading_state: ResMut<NextState<InternalLoadingState<S>>>,
    #[cfg(feature = "progress_tracking")] tracking_id: Res<LoadingStateProgressId<S>>,
    #[cfg(feature = "progress_tracking")] tracker: Option<Res<ProgressTracker<S>>>,
) {
    #[cfg(feature = "progress_tracking")]
    if let Some(tracker) = tracker {
        tracker.set_total(tracking_id.id, 1);
    }
    loading_state.set(InternalLoadingState::LoadingDynamicAssetCollections);
}

pub(crate) fn finish_loading_state<S: FreelyMutableState>(
    state: Res<State<S>>,
    mut next_state: ResMut<NextState<S>>,
    #[cfg(feature = "progress_tracking")] tracking_id: Res<LoadingStateProgressId<S>>,
    #[cfg(feature = "progress_tracking")] tracker: Option<Res<ProgressTracker<S>>>,
    mut loading_state: ResMut<NextState<InternalLoadingState<S>>>,
    asset_loader_configuration: Res<AssetLoaderConfiguration<S>>,
) {
    #[cfg(feature = "progress_tracking")]
    if let Some(tracker) = tracker {
        tracker.set_done(tracking_id.id, 1);
    }
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

pub(crate) fn reset_loading_state<S: FreelyMutableState>(world: &mut World) {
    world.remove_resource::<State<InternalLoadingState<S>>>();
    world.init_resource::<State<InternalLoadingState<S>>>();
}

pub(crate) fn run_loading_state<S: FreelyMutableState>(world: &mut World) {
    let state = world.resource::<State<S>>().get().clone();
    world.run_schedule(LoadingStateSchedule(state));
}

pub fn apply_internal_state_transition<S: FreelyMutableState>(world: &mut World) {
    let next_state = world.remove_resource::<NextState<InternalLoadingState<S>>>();
    match next_state {
        Some(NextState::Pending(entered_state)) => {
            let exited_state = world.remove_resource::<State<InternalLoadingState<S>>>();
            world.insert_resource(State::new(entered_state.clone()));
            trace!(
                "Switching internal state of loading state from {exited_state:?} to {entered_state:?}"
            );
            let state = world.resource::<State<S>>().get().clone();
            if world
                .resource::<Schedules>()
                .contains(OnEnterInternalLoadingState(
                    state.clone(),
                    entered_state.clone(),
                ))
            {
                world.run_schedule(OnEnterInternalLoadingState(state, entered_state));
            }
            world.insert_resource(NextState::<InternalLoadingState<S>>::Unchanged);
        }
        Some(next_state) => world.insert_resource(next_state),
        _ => {}
    }
}
