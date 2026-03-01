use crate::asset_collection::AssetCollection;
use crate::loading::{AssetCollectionFailed, AssetCollectionLoaded, LoadCollectionCommandsExt};
use crate::loading_state::{DynamicPreloadFinished, LoadingStateCoordinator, LoadingStateSpawners};
use bevy_ecs::{
    change_detection::{Res, ResMut},
    observer::On,
    system::Commands,
    world::World,
};
use bevy_log::info;
use bevy_state::state::{FreelyMutableState, NextState, State};
use std::any::type_name;

/// Observer that decrements the coordinator count when a collection finishes loading.
pub(crate) fn on_collection_loaded<S: FreelyMutableState, C: AssetCollection>(
    _: On<AssetCollectionLoaded<C>>,
    current_state: Res<State<S>>,
    mut coordinator: ResMut<LoadingStateCoordinator<S>>,
) {
    let state = current_state.get();
    if let Some(coord) = coordinator.states.get_mut(state) {
        coord.remaining = coord.remaining.saturating_sub(1);
    }
}

/// Observer that decrements the coordinator count and marks failure when a collection fails.
pub(crate) fn on_collection_failed<S: FreelyMutableState, C: AssetCollection>(
    _: On<AssetCollectionFailed<C>>,
    current_state: Res<State<S>>,
    mut coordinator: ResMut<LoadingStateCoordinator<S>>,
) {
    let state = current_state.get();
    if let Some(coord) = coordinator.states.get_mut(state) {
        coord.remaining = coord.remaining.saturating_sub(1);
        coord.any_failed = true;
    }
}

/// Observer that runs when the global dynamic files preload gate completes.
///
/// Spawns the real collection entities and resets the coordinator count to match them.
fn on_preload_finished<S: FreelyMutableState>(
    _: On<AssetCollectionLoaded<DynamicPreloadFinished<S>>>,
    current_state: Res<State<S>>,
    mut commands: Commands,
    spawners: Res<LoadingStateSpawners<S>>,
    mut coordinator: ResMut<LoadingStateCoordinator<S>>,
) {
    let state = current_state.get();
    let per_state = spawners
        .states
        .get(state)
        .expect("No spawners for current loading state when DynamicPreloadFinished fired");
    let coord = coordinator.states.entry(state.clone()).or_default();
    coord.remaining = per_state.collection_spawners.len();
    for spawn in per_state.collection_spawners.iter() {
        spawn(&mut commands);
    }
}

/// Spawns loading entities when entering the loading state.
///
/// Runs in `OnEnter(loading_state)`. If there are global dynamic asset files to load,
/// a [`DynamicPreloadFinished<S>`] gate entity is spawned first; its observer then spawns
/// the real collection entities once all files are registered. Otherwise, collection entities
/// are spawned directly.
pub(crate) fn enter_loading_state<S: FreelyMutableState>(
    current_state: Res<State<S>>,
    mut commands: Commands,
    spawners: Res<LoadingStateSpawners<S>>,
    mut coordinator: ResMut<LoadingStateCoordinator<S>>,
) {
    let state = current_state.get().clone();
    info!("Entering loading state '{:?}'", &state);

    let coord = coordinator.states.entry(state.clone()).or_default();
    coord.any_failed = false;
    coord.completed = false;

    let per_state = spawners.states.get(&state).expect(
        "No spawners registered for the current loading state. Was add_loading_state called?",
    );

    if per_state.global_dynamic_files.is_empty() {
        coord.remaining = per_state.collection_spawners.len();
        for spawn in per_state.collection_spawners.iter() {
            spawn(&mut commands);
        }
    } else {
        // Gate: spawn a preload entity that carries all global dynamic file specs.
        // Its observer fires after all files are loaded, then spawns the real collections.
        coord.remaining = 1;
        let mut preload = commands.load_collection::<DynamicPreloadFinished<S>>();
        for spec in per_state.global_dynamic_files.iter().cloned() {
            preload.push_dynamic_file_spec(spec);
        }
        preload.observe(on_preload_finished::<S>);
        preload.observe(on_collection_failed::<S, DynamicPreloadFinished<S>>);
    }
}

/// Checks if all collections have finished and drives the user state transition.
///
/// Runs in `Update` in [`LoadingStateSet`](crate::loading_state::LoadingStateSet),
/// after [`AssetLoadingSet`](crate::loading::AssetLoadingSet).
pub(crate) fn check_loading_coordinator<S: FreelyMutableState>(world: &mut World) {
    let state = world.resource::<State<S>>().get().clone();

    // Early exit: not done yet, or already completed
    {
        let coordinator = world.resource::<LoadingStateCoordinator<S>>();
        match coordinator.states.get(&state) {
            Some(coord) if !coord.completed && coord.remaining == 0 => {
                // Fall through to handle completion
            }
            _ => return,
        }
    }

    let any_failed = world
        .resource::<LoadingStateCoordinator<S>>()
        .states
        .get(&state)
        .map(|c| c.any_failed)
        .unwrap_or(false);

    if any_failed {
        let failure = world
            .resource::<LoadingStateSpawners<S>>()
            .states
            .get(&state)
            .and_then(|s| s.failure_state.clone());
        if let Some(failure) = failure {
            world.resource_mut::<NextState<S>>().set(failure);
        }
    } else {
        // Temporarily remove LoadingStateSpawners so callbacks can freely access &mut World
        if let Some(spawners) = world.remove_resource::<LoadingStateSpawners<S>>() {
            let next_state = if let Some(per_state) = spawners.states.get(&state) {
                for cb in &per_state.finally_callbacks {
                    cb(world);
                }
                per_state.next_state.clone()
            } else {
                None
            };
            world.insert_resource(spawners);
            if let Some(next) = next_state {
                world.resource_mut::<NextState<S>>().set(next);
            }
        }
    }

    world
        .resource_mut::<LoadingStateCoordinator<S>>()
        .states
        .entry(state)
        .or_default()
        .completed = true;

    info!("Loading state for '{}' completed", type_name::<S>());
}
