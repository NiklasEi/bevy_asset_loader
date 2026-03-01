//! Asset collection loading using commands and observers.
//!
//! This module provides the core loading machinery used both by [`LoadingState`] and
//! directly by users who want to load collections without a dedicated loading state.
//!
//! Trigger collection loading imperatively via [`LoadCollectionCommandsExt`], then react
//! to completion with an observer on [`AssetCollectionLoaded`]:
//!
//! ```edition2024
//! # use bevy_asset_loader::prelude::*;
//! # use bevy::prelude::*;
//! # use bevy::asset::AssetPlugin;
//! fn main() {
//!     App::new()
//! # /*
//!         .add_plugins(DefaultPlugins)
//! # */
//! #       .add_plugins((MinimalPlugins, AssetPlugin::default()))
//!         .add_plugins(AssetLoadingPlugin)
//!         .add_systems(Startup, setup)
//! #       .set_runner(|mut app| { app.update(); AppExit::Success })
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.load_collection::<AudioAssets>();
//! }
//!
//! #[derive(AssetCollection, Resource)]
//! struct AudioAssets {
//!     #[asset(path = "audio/background.ogg")]
//!     background: Handle<AudioSource>,
//! }
//! ```
//!
//! [`LoadingState`]: crate::loading_state::LoadingState

use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use bevy_app::{App, Plugin, Update};
use bevy_asset::{Asset, AssetServer, Assets, LoadState, UntypedHandle};
use bevy_ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    event::{EntityEvent, EntityTrigger, Event},
    query::{Added, With, Without},
    schedule::{IntoScheduleConfigs, SystemSet},
    system::{Command, Commands, IntoObserverSystem},
    world::World,
};

use crate::asset_collection::AssetCollection;
use crate::dynamic_asset::{DynamicAssetCollection, DynamicAssets};

#[cfg(feature = "standard_dynamic_assets")]
use crate::standard_dynamic_asset::{
    StandardDynamicAssetArrayCollection, StandardDynamicAssetCollection,
};
#[cfg(feature = "standard_dynamic_assets")]
use bevy_common_assets::ron::RonAssetPlugin;

/// System set containing the asset collection loading systems.
///
/// Systems that need to run after collections have been polled should use
/// `.after(AssetLoadingSet)`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetLoadingSet;

/// Plugin that enables asset collection loading via [`Commands`].
///
/// This plugin is automatically added by [`LoadingState`]. Add it explicitly only if you
/// are loading collections without a loading state.
///
/// [`LoadingState`]: crate::loading_state::LoadingState
pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicAssets>();

        #[cfg(feature = "standard_dynamic_assets")]
        if !app.is_plugin_added::<RonAssetPlugin<StandardDynamicAssetCollection>>() {
            app.add_plugins(RonAssetPlugin::<StandardDynamicAssetCollection>::new(&[
                "assets.ron",
            ]));
        }
        #[cfg(feature = "standard_dynamic_assets")]
        if !app.is_plugin_added::<RonAssetPlugin<StandardDynamicAssetArrayCollection>>() {
            app.add_plugins(RonAssetPlugin::<StandardDynamicAssetArrayCollection>::new(
                &[],
            ));
        }

        app.add_systems(
            Update,
            (
                check_dynamic_files,
                check_asset_handles,
                process_failed_collections,
            )
                .chain()
                .in_set(AssetLoadingSet),
        );
    }
}

/// Triggered when an [`AssetCollection`] has finished loading.
///
/// This event is entity-scoped: observers attached to the loading entity via
/// [`LoadCollectionCommands::observe`] will receive it, as will global observers.
pub struct AssetCollectionLoaded<C: AssetCollection> {
    /// The entity used for this loading job (now despawned).
    pub entity: Entity,
    _marker: PhantomData<C>,
}

impl<C: AssetCollection + 'static> Event for AssetCollectionLoaded<C> {
    type Trigger<'a> = EntityTrigger;
}

impl<C: AssetCollection + 'static> EntityEvent for AssetCollectionLoaded<C> {
    fn event_target(&self) -> Entity {
        self.entity
    }
}

/// Triggered when an [`AssetCollection`] fails to load.
///
/// This event is entity-scoped: observers attached to the loading entity via
/// [`LoadCollectionCommands::observe`] will receive it, as will global observers.
pub struct AssetCollectionFailed<C: AssetCollection> {
    /// The entity used for this loading job (now despawned).
    pub entity: Entity,
    _marker: PhantomData<C>,
}

impl<C: AssetCollection + 'static> Event for AssetCollectionFailed<C> {
    type Trigger<'a> = EntityTrigger;
}

impl<C: AssetCollection + 'static> EntityEvent for AssetCollectionFailed<C> {
    fn event_target(&self) -> Entity {
        self.entity
    }
}

/// Extension trait for [`Commands`] enabling asset collection loading.
pub trait LoadCollectionCommandsExt {
    /// Start loading an [`AssetCollection`].
    ///
    /// Returns a [`LoadCollectionCommands`] builder for attaching observers and
    /// configuring dynamic asset files. Loading begins when the builder is dropped.
    fn load_collection<C: AssetCollection>(&mut self) -> LoadCollectionCommands<'_, C>;
}

impl LoadCollectionCommandsExt for Commands<'_, '_> {
    fn load_collection<C: AssetCollection>(&mut self) -> LoadCollectionCommands<'_, C> {
        let entity = self.spawn(CollectionLoadingMarker).id();
        LoadCollectionCommands {
            entity,
            commands: self.reborrow(),
            dynamic_files: vec![],
            _marker: PhantomData,
        }
    }
}

/// Builder returned from [`LoadCollectionCommandsExt::load_collection`].
///
/// The loading job starts when this value is dropped.
pub struct LoadCollectionCommands<'w, C: AssetCollection> {
    entity: Entity,
    commands: Commands<'w, 'w>,
    dynamic_files: Vec<DynamicFileSpec>,
    _marker: PhantomData<C>,
}

impl<'w, C: AssetCollection> LoadCollectionCommands<'w, C> {
    /// Returns the ECS entity backing this loading job.
    pub fn id(&self) -> Entity {
        self.entity
    }

    /// Queue a dynamic assets file to be loaded before the collection starts loading.
    ///
    /// The file is loaded as asset type `D`. Once loaded, its keys are registered
    /// in the global [`DynamicAssets`] resource, then the collection is loaded.
    ///
    /// Requires the asset plugin for `D` to already be registered (e.g.
    /// [`bevy_common_assets::ron::RonAssetPlugin`] for [`StandardDynamicAssetCollection`]).
    /// [`AssetLoadingPlugin`] registers it automatically when the
    /// `standard_dynamic_assets` feature is enabled.
    #[must_use]
    pub fn with_dynamic_assets_file<D: DynamicAssetCollection + Asset>(
        &mut self,
        path: &str,
    ) -> &mut Self {
        let path = path.to_string();
        self.dynamic_files.push(DynamicFileSpec {
            load_fn: Arc::new(move |asset_server| asset_server.load::<D>(path.clone()).untyped()),
            register_fn: Arc::new(move |handle: UntypedHandle, world: &mut World| {
                let typed_handle = handle.typed::<D>();
                let mut dynamic_assets =
                    world.remove_resource::<DynamicAssets>().unwrap_or_default();
                {
                    let assets = world.resource::<Assets<D>>();
                    if let Some(collection) = assets.get(&typed_handle) {
                        collection.register(&mut dynamic_assets);
                    }
                }
                world.insert_resource(dynamic_assets);
            }),
        });
        self
    }

    /// Push a pre-built [`DynamicFileSpec`] onto this loading job.
    ///
    /// Used internally by the state-based loading pipeline.
    pub(crate) fn push_dynamic_file_spec(&mut self, spec: DynamicFileSpec) -> &mut Self {
        self.dynamic_files.push(spec);
        self
    }

    /// Attach an observer to the loading entity.
    ///
    /// The observer will be triggered for entity-scoped events on this entity, such
    /// as [`AssetCollectionLoaded<C>`] and [`AssetCollectionFailed<C>`].
    pub fn observe<E: EntityEvent, B: Bundle, M>(
        &mut self,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        self.commands.entity(self.entity).observe(observer);
        self
    }
}

impl<C: AssetCollection> Drop for LoadCollectionCommands<'_, C> {
    fn drop(&mut self) {
        let entity = self.entity;
        let files = mem::take(&mut self.dynamic_files);
        self.commands.queue(StartLoadingCollection::<C> {
            entity,
            dynamic_files: files,
            _marker: PhantomData,
        });
    }
}

type RegisterFn = Arc<dyn Fn(UntypedHandle, &mut World) + Send + Sync>;
type StartLoadingFn = Box<dyn Fn(&mut World) -> Vec<UntypedHandle> + Send + Sync>;
type WorldCallbackFn = Box<dyn Fn(&mut World) + Send + Sync>;
type EntityWorldCallbackFn = Box<dyn Fn(Entity, &mut World) + Send + Sync>;

#[derive(Component)]
struct CollectionLoadingMarker;

#[derive(Component)]
struct CollectionHandles(Vec<UntypedHandle>);

struct DynamicFileEntry {
    handle: UntypedHandle,
    register_fn: RegisterFn,
}

#[derive(Component)]
struct DynamicFileEntries {
    entries: Vec<DynamicFileEntry>,
}

#[derive(Component)]
struct CollectionCallbacks {
    start_loading: StartLoadingFn,
    create_and_insert: WorldCallbackFn,
    trigger_loaded: EntityWorldCallbackFn,
    trigger_failed: EntityWorldCallbackFn,
}

#[derive(Component)]
struct CollectionLoadingFailed;

/// Specification for loading a dynamic asset collection file.
///
/// Stores an `Arc`-wrapped load function (callable multiple times across state re-entries)
/// and a register function to apply the loaded collection to [`DynamicAssets`].
#[derive(Clone)]
pub(crate) struct DynamicFileSpec {
    pub(crate) load_fn: Arc<dyn Fn(&AssetServer) -> UntypedHandle + Send + Sync>,
    pub(crate) register_fn: RegisterFn,
}

struct StartLoadingCollection<C: AssetCollection> {
    entity: Entity,
    dynamic_files: Vec<DynamicFileSpec>,
    _marker: PhantomData<C>,
}

impl<C: AssetCollection> Command for StartLoadingCollection<C> {
    fn apply(self, world: &mut World) {
        world.init_resource::<DynamicAssets>();

        let callbacks = CollectionCallbacks {
            start_loading: Box::new(|world| C::load(world)),
            create_and_insert: Box::new(|world| {
                let collection = C::create(world);
                world.insert_resource(collection);
            }),
            trigger_loaded: Box::new(|entity, world| {
                world.trigger(AssetCollectionLoaded::<C> {
                    entity,
                    _marker: PhantomData,
                });
            }),
            trigger_failed: Box::new(|entity, world| {
                world.trigger(AssetCollectionFailed::<C> {
                    entity,
                    _marker: PhantomData,
                });
            }),
        };

        if self.dynamic_files.is_empty() {
            let handles = C::load(world);
            world
                .entity_mut(self.entity)
                .insert((CollectionHandles(handles), callbacks));
        } else {
            let entries: Vec<DynamicFileEntry> = {
                let asset_server = world.resource::<AssetServer>();
                self.dynamic_files
                    .into_iter()
                    .map(|spec| DynamicFileEntry {
                        handle: (spec.load_fn)(asset_server),
                        register_fn: spec.register_fn,
                    })
                    .collect()
            };
            world
                .entity_mut(self.entity)
                .insert((DynamicFileEntries { entries }, callbacks));
        }
    }
}

fn check_dynamic_files(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, (With<DynamicFileEntries>, Without<CollectionLoadingFailed>)>()
        .iter(world)
        .collect();

    for entity in entities {
        let (all_loaded, any_failed) = {
            let entity_ref = world.entity(entity);
            let files = entity_ref.get::<DynamicFileEntries>().unwrap();
            let asset_server = world.resource::<AssetServer>();

            let any_failed = files.entries.iter().any(|entry| {
                matches!(
                    asset_server.get_load_state(entry.handle.id()),
                    Some(LoadState::Failed(_))
                )
            });
            let all_loaded = files.entries.iter().all(|entry| {
                matches!(
                    asset_server.get_load_state(entry.handle.id()),
                    Some(LoadState::Loaded)
                )
            });
            (all_loaded, any_failed)
        };

        if any_failed {
            world.entity_mut(entity).insert(CollectionLoadingFailed);
        } else if all_loaded {
            // Take the dynamic files component out so we can use &mut world freely.
            let files = world
                .entity_mut(entity)
                .take::<DynamicFileEntries>()
                .unwrap();

            for entry in &files.entries {
                (entry.register_fn)(entry.handle.clone(), world);
            }

            // Take callbacks, call start_loading, then re-insert.
            let callbacks = world
                .entity_mut(entity)
                .take::<CollectionCallbacks>()
                .unwrap();
            let handles = (callbacks.start_loading)(world);
            world
                .entity_mut(entity)
                .insert((callbacks, CollectionHandles(handles)));
        }
    }
}

fn check_asset_handles(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, (
            With<CollectionHandles>,
            Without<DynamicFileEntries>,
            Without<CollectionLoadingFailed>,
        )>()
        .iter(world)
        .collect();

    for entity in entities {
        let status = {
            let entity_ref = world.entity(entity);
            let handles = entity_ref.get::<CollectionHandles>().unwrap();
            let asset_server = world.resource::<AssetServer>();

            let any_failed = handles.0.iter().any(|handle| {
                asset_server
                    .get_recursive_dependency_load_state(handle.id())
                    .map(|s| s.is_failed())
                    .unwrap_or(false)
            });
            let all_loaded = handles
                .0
                .iter()
                .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));

            if any_failed {
                LoadStatus::Failed
            } else if all_loaded {
                LoadStatus::Done
            } else {
                LoadStatus::Loading
            }
        };

        match status {
            LoadStatus::Failed => {
                world.entity_mut(entity).insert(CollectionLoadingFailed);
            }
            LoadStatus::Done => {
                let callbacks = world
                    .entity_mut(entity)
                    .take::<CollectionCallbacks>()
                    .unwrap();
                (callbacks.create_and_insert)(world);
                (callbacks.trigger_loaded)(entity, world);
                world.despawn(entity);
            }
            LoadStatus::Loading => {}
        }
    }
}

fn process_failed_collections(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, (With<CollectionCallbacks>, Added<CollectionLoadingFailed>)>()
        .iter(world)
        .collect();

    for entity in entities {
        let callbacks = world
            .entity_mut(entity)
            .take::<CollectionCallbacks>()
            .unwrap();
        (callbacks.trigger_failed)(entity, world);
        world.despawn(entity);
    }
}

enum LoadStatus {
    Loading,
    Done,
    Failed,
}
