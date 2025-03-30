use std::{any::type_name, marker::PhantomData};

use bevy::{
    asset::{AssetServer, UntypedHandle},
    ecs::system::{SystemId, SystemState},
    log::warn,
    prelude::{
        Commands, Component, Entity, NextState, Query, Res, ResMut, Result, With, Without, World,
    },
    state::state::{FreelyMutableState, State},
};

use crate::{loading_state::AssetLoaderConfiguration, prelude::AssetCollection};

#[derive(Component)]
pub struct AssetCollectionNode {
    name: String,
    add: SystemId,
    load: SystemId<(), Vec<UntypedHandle>>,
}

impl AssetCollectionNode {
    fn new<C: AssetCollection>(world: &mut World) -> Self {
        let add = world.register_system(C::add);
        let load = world.register_system(C::load);
        let name = type_name::<C>().to_owned();

        AssetCollectionNode { add, load, name }
    }
}

#[derive(Component)]
pub struct LoadingHandles {
    handles: Vec<UntypedHandle>,
}

pub fn start_loading_collection<S: FreelyMutableState>(
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

#[derive(Component)]
pub struct LoadingStateMarker<S> {
    _marker: PhantomData<S>,
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
            if let Some(tracker) = progress_tracker {
                tracker.set_progress(entry_id, done, total);
            }
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
