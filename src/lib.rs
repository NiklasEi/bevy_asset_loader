//! # Bevy asset loader
//!
//! **WIP!**
//!
//! The goal of this crate is to offer an easy way for bevy games to load all their assets.
//!
//! ```edition2018
//! # use bevy_assets_loader::{AssetLoaderPlugin, AssetCollection};
//! # use bevy::prelude::*;
//! fn no_assets() {
//!     App::build()
//!         .add_state(MyStates::Load)
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(AssetLoaderPlugin::new(MyStates::Load, MyStates::Next).with_collection::<MyAssets>())
//!         .run();
//! }
//!
//! #[derive(AssetCollection)]
//! struct MyAssets {
//!     #[path = "textures/ground.png"]
//!     ground: Handle<Texture>,
//!     #[path = "walking.ogg"]
//!     walking_sound: Handle<AudioSource>
//! }
//!
//! #[derive(Clone, Eq, PartialEq, Debug, Hash)]
//! enum MyStates {
//!     Load,
//!     Next
//! }
//! ```
//!

use bevy::app::{AppBuilder, Plugin};
use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::prelude::State;
use bevy::ecs::schedule::SystemDescriptor;
use bevy::ecs::system::IntoSystem;
use bevy::prelude::{Commands, Res, ResMut, SystemSet};
use std::fmt::Debug;
use std::hash::Hash;

pub struct AssetLoaderPlugin<T> {
    on: T,
    next: T,
    systems: Vec<Box<dyn FnOnce() -> dyn Into<SystemDescriptor>>>,
}

impl<T> AssetLoaderPlugin<T>
where
    T: Component + Debug + Clone + Eq + Hash,
{
    pub fn new(on: T, next: T) -> AssetLoaderPlugin<T> {
        Self {
            on,
            next,
            systems: vec![],
        }
    }

    pub fn with_collection<A: AssetCollection>(&mut self) -> &mut Self {
        self.systems
            .push(Box::new(|| insert_asset_collection::<A>.system()));

        self
    }
}

// ToDo
pub trait AssetCollection {
    fn create(asset_server: &mut ResMut<AssetServer>) -> Self;
}

struct AssetLoaderNextState<T> {
    next: T,
}

impl<T> Plugin for AssetLoaderPlugin<T>
where
    T: Component + Debug + Clone + Eq + Hash,
{
    fn build(&self, app: &mut AppBuilder) {
        let mut insert_resources_set = SystemSet::on_exit(self.on.clone());
        self.systems.iter().map(|system| {
            insert_resources_set.with_system(system());
        });

        app.insert_resource(AssetLoaderNextState::<T> {
            next: self.next.clone(),
        })
        .add_system_set(
            SystemSet::on_enter(self.on.clone()).with_system(start_loading::<T>.system()),
        )
        .add_system_set(insert_resources_set);
    }
}

fn insert_asset_collection<T: AssetCollection>(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
) {
    commands.insert_resource(T::create(&mut asset_server));
}

fn start_loading<T: Component + Debug + Clone + Eq + Hash>(
    mut state: ResMut<State<T>>,
    next_state: Res<AssetLoaderNextState<T>>,
) {
    println!("loading");
    state
        .set(next_state.next.clone())
        .expect("Failed to set next State");
}
