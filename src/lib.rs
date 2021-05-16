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
//!         .add_plugin(AssetLoaderPlugin::new(MyStates::Load, MyStates::Next))
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

use bevy::app::{Plugin, AppBuilder};
use bevy::ecs::component::Component;
use std::fmt::Debug;
use std::hash::Hash;
use bevy::prelude::{SystemSet, ResMut, Res};
use bevy::ecs::prelude::State;
use bevy::ecs::system::IntoSystem;
use std::collections::VecDeque;

pub struct AssetLoaderPlugin<T> {
    on: T,
    next: T
}

impl <T> AssetLoaderPlugin<T>
where T: Component + Debug + Clone + Eq + Hash {
    pub fn new(on: T, next: T) -> AssetLoaderPlugin<T> {
        Self {
            on, next
        }
    }
}

// ToDo
pub trait AssetCollection {
    fn get_keys() -> Vec<String>;
    fn get_path(key: String) -> String;
}

struct AssetLoaderNextState<T> {
    next: T
}

impl <T> Plugin for AssetLoaderPlugin<T>
    where T: Component + Debug + Clone + Eq + Hash {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(AssetLoaderNextState::<T> {
            next: self.next.clone()
        })
            .add_system_set(SystemSet::on_enter(self.on.clone())
            .with_system(start_loading::<T>.system()));
    }
}

fn start_loading<T: Component + Debug + Clone + Eq + Hash>(mut state: ResMut<State<T>>, next_state: Res<AssetLoaderNextState<T>>) {
    println!("loading");
    state.set(next_state.next.clone()).expect("Failed to set next State");
}
