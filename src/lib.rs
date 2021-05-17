//! # Bevy asset loader
//!
//! **WIP!**
//!
//! The goal of this crate is to offer an easy way for bevy games to load all their assets.
//!
//! ```edition2018
//! # use bevy_assets_loader::{AssetLoaderPlugin, AssetCollection};
//! # use bevy::prelude::*;
//! fn main() {
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
use std::marker::PhantomData;

pub struct AssetLoaderPlugin<A, T> {
    on: T,
    next: T,
    marker: PhantomData<A>,
}

impl<A, T> AssetLoaderPlugin<A, T>
where
    A: AssetCollection,
    T: Component + Debug + Clone + Eq + Hash,
{
    pub fn new(on: T, next: T) -> AssetLoaderPlugin<A, T> {
        Self {
            on,
            next,
            marker: PhantomData,
        }
    }
}

pub trait AssetCollection: Component {
    fn create(asset_server: &mut ResMut<AssetServer>) -> Self;
}

struct AssetLoaderNextState<T> {
    next: T,
}

impl<Assets, State> Plugin for AssetLoaderPlugin<Assets, State>
where
    Assets: AssetCollection,
    State: Component + Debug + Clone + Eq + Hash,
{
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(AssetLoaderNextState::<State> {
            next: self.next.clone(),
        })
        .add_system_set(
            SystemSet::on_enter(self.on.clone()).with_system(start_loading::<Assets>.system()),
        )
        .add_system_set(
            SystemSet::on_update(self.on.clone())
                .with_system(check_loading_state::<State, Assets>.system()),
        )
        .add_system_set(
            SystemSet::on_exit(self.on.clone())
                .with_system(insert_asset_collection::<Assets>.system()),
        );
    }
}

fn start_loading<Assets: AssetCollection>() {
    // Todo
}

fn check_loading_state<T: Component + Debug + Clone + Eq + Hash, A: AssetCollection>(
    mut state: ResMut<State<T>>,
    next_state: Res<AssetLoaderNextState<T>>,
) {
    // todo
    state
        .set(next_state.next.clone())
        .expect("Failed to set next State");
}

fn insert_asset_collection<A: AssetCollection>(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
) {
    commands.insert_resource(A::create(&mut asset_server));
}
