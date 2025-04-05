#[cfg(feature = "progress_tracking")]
use crate::loading_state::{AssetCollectionsProgressId, LoadingStateProgressId};
use crate::loading_state::{
    AssetLoaderConfiguration, InternalLoadingState, LoadingAssetHandles, LoadingStateSchedule,
    OnEnterInternalLoadingState,
};
use crate::rewrite::{AssetCollectionNode, LoadingHandles, LoadingStateMarker};
use bevy::asset::{AssetServer, UntypedHandle};
use bevy::ecs::system::{SystemId, SystemState};
use bevy::ecs::world::{FromWorld, World};
use bevy::log::{debug, info, trace, warn};
use bevy::prelude::{
    Commands, Entity, NextState, Query, Res, ResMut, Resource, Result, Schedules, With, Without,
};
use bevy::state::state::{FreelyMutableState, State};
#[cfg(feature = "progress_tracking")]
use iyes_progress::{ProgressEntryId, ProgressTracker};
use std::any::{TypeId, type_name};
use std::marker::PhantomData;

pub(crate) fn finally_init_resource<Asset: Resource + FromWorld>(world: &mut World) {
    world.init_resource::<Asset>();
}

pub fn start_loading_collection(
    world: &mut World,
    system_state: &mut SystemState<Query<(Entity, &AssetCollectionNode), Without<LoadingHandles>>>,
) -> Result {
    let nodes = system_state.get_mut(world);

    let system_ids: Vec<(Entity, SystemId<(), Vec<UntypedHandle>>)> = nodes
        .iter()
        .map(|(entity, collection)| (entity, collection.load))
        .collect();

    for (entity, load) in system_ids {
        let handles = world.run_system(load)?;
        world.entity_mut(entity).insert(LoadingHandles { handles });
    }

    Ok(())
}

pub fn check_progress<S: FreelyMutableState>(
    collections: Query<
        (Entity, &AssetCollectionNode, &LoadingHandles),
        With<LoadingStateMarker<S>>,
    >,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut asset_loader_config: ResMut<AssetLoaderConfiguration<S>>,
    state: Res<State<S>>,
    mut next_state: ResMut<NextState<S>>,

    #[cfg(feature = "progress_tracking")] progress_id: Res<LoadingStateProgressId<S>>,
    #[cfg(feature = "progress_tracking")] progress_tracker: Option<Res<ProgressTracker<S>>>,
) {
    let Some(config) = asset_loader_config
        .state_configurations
        .get_mut(state.get())
    else {
        warn!(
            "Failed to read loading state configuration in check_progress<{}>",
            type_name::<S>()
        );
        return;
    };

    #[cfg(feature = "progress_tracking")]
    let Some(tracker) = progress_tracker else {
        warn!(
            "Failed to get progress tracker in check_progress<{}>",
            type_name::<S>()
        );
        return;
    };
    let mut count = collections.iter().count();
    let mut failed = false;
    for (entity, collection, loading_handles) in &collections {
        failed = failed
            || loading_handles.handles.iter().any(|handle| {
                asset_server
                    .get_recursive_dependency_load_state(handle.id())
                    .map(|state| state.is_failed())
                    .unwrap_or(false)
            });

        let total = loading_handles.handles.len();

        let done = loading_handles
            .handles
            .iter()
            .filter(|handle| asset_server.is_loaded_with_dependencies(handle.id()))
            .count();

        #[cfg(feature = "progress_tracking")]
        {
            let entry_id = progress_id.id;
            tracker.set_progress(entry_id, done as u32, total as u32);
        }
        if total == done {
            count -= 1;
            commands.run_system(collection.add);
            commands.entity(entity).despawn();
        }
    }
    if failed && config.failure.is_some() {
        let failure = config.failure.clone().unwrap();
        next_state.set(failure);
    }
    if count == 0 && config.next.is_some() {
        let next = config.next.clone().unwrap();
        next_state.set(next);
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
