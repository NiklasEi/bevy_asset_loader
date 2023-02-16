use bevy::utils::HashMap;
use std::any::TypeId;
use std::fmt::Debug;

use bevy::asset::{AssetServer, HandleUntyped};
use bevy::ecs::system::Resource;
use bevy::ecs::world::World;
use bevy::prelude::States;
use std::marker::PhantomData;

/// Different typed that can generate the asset field value of a dynamic asset
pub enum DynamicAssetType {
    /// Dynamic asset that is defined by a single handle
    Single(HandleUntyped),
    /// Dynamic asset that is defined by multiple handles
    Collection(Vec<HandleUntyped>),
}

/// Any type implementing this trait can be assigned to asset keys as part of a dynamic
/// asset collection.
pub trait DynamicAsset: Debug + Send + Sync {
    /// Return handles to all required asset paths
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped>;

    /// Return the handle(s) defining this asset
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error>;
}

/// Resource to dynamically resolve keys to assets.
///
/// This resource is set by a [`LoadingState`](crate::loading_state::LoadingState) and is read when entering the corresponding Bevy [`State`](::bevy::ecs::schedule::State).
/// If you want to manage your dynamic assets manually, they should be configured in a previous [`State`](::bevy::ecs::schedule::State).
///
/// See the `manual_dynamic_asset` example.
#[derive(Resource, Default)]
pub struct DynamicAssets {
    key_asset_map: HashMap<String, Box<dyn DynamicAsset>>,
}

impl DynamicAssets {
    /// Get the asset corresponding to the given key.
    pub fn get_asset(&self, key: &str) -> Option<&dyn DynamicAsset> {
        self.key_asset_map.get(key).map(|boxed| boxed.as_ref())
    }

    /// Set the corresponding dynamic asset for the given key.
    ///
    /// In case the key is already known, its value will be overwritten.
    pub fn register_asset<K: Into<String>>(&mut self, key: K, asset: Box<dyn DynamicAsset>) {
        self.key_asset_map.insert(key.into(), asset);
    }
}

/// This traits describes types that contain asset configurations and can
/// register them in the [`DynamicAssets`] resource.
pub trait DynamicAssetCollection {
    /// Register all dynamic assets inside the collection in the [`DynamicAssets`] resource.
    fn register(&self, dynamic_assets: &mut DynamicAssets);
}

/// Resource keeping track of dynamic asset collection files for different loading states
#[derive(Resource, Debug)]
pub struct DynamicAssetCollections<State: States> {
    /// Dynamic asset collection files for different loading states.
    ///
    /// The file lists get loaded and emptied at the beginning of the loading states.
    /// Make sure to add any file you would like to load before entering the loading state!
    pub files: HashMap<State, HashMap<TypeId, Vec<String>>>,
    pub(crate) _marker: PhantomData<State>,
}

impl<State: States> Default for DynamicAssetCollections<State> {
    fn default() -> Self {
        DynamicAssetCollections {
            files: HashMap::default(),
            _marker: PhantomData::default(),
        }
    }
}
