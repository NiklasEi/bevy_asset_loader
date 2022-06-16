//! The goal of this crate is to offer an easy way for bevy games to load all their assets in a loading [`State`](::bevy::ecs::schedule::State).
//!
//! `bevy_asset_loader` introduces the derivable trait [`AssetCollection`](crate::asset_collection::AssetCollection). Structs with asset handles
//! can be automatically loaded during a configurable loading [`State`](::bevy::ecs::schedule::State). Afterwards they will be inserted as
//! resources containing loaded handles and the plugin will switch to a second configurable [`State`](::bevy::ecs::schedule::State).
//!
//! ```edition2021
//! # use bevy_asset_loader::prelude::*;
//! # use bevy::prelude::*;
//! # use bevy::asset::AssetPlugin;
//! #
//! # #[cfg(feature="stageless")]
//! # use iyes_loopless::prelude::*;
//!
//! # #[cfg(not(feature="stageless"))]
//! fn main() {
//!     App::new()
//! # /*
//!         .add_plugins(DefaultPlugins)
//! # */
//! #       .add_plugins(MinimalPlugins)
//! #       .init_resource::<iyes_progress::ProgressCounter>()
//! #       .add_plugin(AssetPlugin::default())
//!         .add_loading_state(
//!             LoadingState::new(GameState::Loading)
//!                 .continue_to_state(GameState::Next)
//!                 .with_collection::<AudioAssets>()
//!                 .with_collection::<ImageAssets>()
//!         )
//!         .add_state(GameState::Loading)
//!         .add_system_set(SystemSet::on_update(GameState::Next)
//!             .with_system(use_asset_handles)
//!         )
//! #       .set_runner(|mut app| app.schedule.run(&mut app.world))
//!         .run();
//! }
//!
//! # #[cfg(all(feature="stageless"))]
//! # fn main() {
//! #     App::new()
//! #       .add_loopless_state(GameState::Loading)
//! # /*
//!         .add_plugins(DefaultPlugins)
//! # */
//! #       .add_plugins(MinimalPlugins)
//! #       .init_resource::<iyes_progress::ProgressCounter>()
//! #       .add_plugin(AssetPlugin::default())
//! #       .add_loading_state(
//! #         LoadingState::new(GameState::Loading)
//! #           .continue_to_state(GameState::Next)
//! #           .with_collection::<AudioAssets>()
//! #           .with_collection::<ImageAssets>()
//! #       )
//! #       .add_system(use_asset_handles.run_in_state(GameState::Next))
//! #       .set_runner(|mut app| app.schedule.run(&mut app.world))
//! #       .run();
//! }
//!
//! #[derive(AssetCollection)]
//! struct AudioAssets {
//!     #[asset(path = "audio/background.ogg")]
//!     background: Handle<AudioSource>,
//!     #[asset(path = "audio/plop.ogg")]
//!     plop: Handle<AudioSource>
//! }
//!
//! #[derive(AssetCollection)]
//! pub struct ImageAssets {
//!     #[asset(path = "images/player.png")]
//!     pub player: Handle<Image>,
//!     #[asset(path = "images/tree.png")]
//!     pub tree: Handle<Image>,
//! }
//!
//! // since this function runs in MyState::Next, we know our assets are loaded.
//! // We can get their handles from the AudioAssets resource.
//! fn use_asset_handles(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
//!     audio.play(audio_assets.background.clone());
//! }
//!
//! #[derive(Clone, Eq, PartialEq, Debug, Hash)]
//! enum GameState {
//!     Loading,
//!     Next
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Trait definition for types that represent a collection of assets
///
/// And extension traits to insert said collections into your Bevy app or world
pub mod asset_collection;
/// Types and infrastructure to load and use dynamic assets
pub mod dynamic_asset;
/// A game state responsible for loading assets
pub mod loading_state;
/// Dynamic assets for common Bevy asset types
#[cfg_attr(docsrs, doc(cfg(feature = "dynamic_assets")))]
#[cfg(feature = "dynamic_assets")]
pub mod standard_dynamic_asset;

#[doc(hidden)]
pub mod prelude {
    #[doc(hidden)]
    #[cfg(feature = "dynamic_assets")]
    pub use crate::standard_dynamic_asset::{StandardDynamicAsset, StandardDynamicAssetCollection};
    #[doc(hidden)]
    pub use crate::{
        asset_collection::{AssetCollection, AssetCollectionApp, AssetCollectionWorld},
        dynamic_asset::{
            DynamicAsset, DynamicAssetCollection, DynamicAssetCollections, DynamicAssetType,
            DynamicAssets,
        },
        loading_state::{LoadingState, LoadingStateAppExt},
    };
}

#[cfg(all(feature = "2d", feature = "3d"))]
#[doc = include_str!("../../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
