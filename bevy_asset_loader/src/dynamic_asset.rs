use bevy::utils::HashMap;
use std::any::TypeId;
use std::fmt::Debug;

use bevy::asset::{Asset, AssetServer, HandleUntyped};
use bevy::ecs::schedule::States;
use bevy::ecs::system::Resource;
use bevy::ecs::world::World;
use std::marker::PhantomData;

/// Different typed that can generate the asset field value of a dynamic asset
pub enum DynamicAssetType {
    /// Dynamic asset that is defined by a single handle
    Single(HandleUntyped),
    /// Dynamic asset that is defined by multiple handles
    Collection(Vec<HandleUntyped>),
}

/// Untracked asset handles that a dynamic asset key resolves to
pub enum UntrackedDynamicAssetType {
    /// Dynamic asset that is defined by a single handle
    Single(HandleId),
    /// Dynamic asset that is defined by multiple handles
    Collection(Vec<HandleId>),
}

/// Cached untracked asset handles for all dynamic assets that have been loaded
///
/// This cache is not cleaned up and can contain unloaded handles!
pub struct DynamicAssetCache {
    key_asset_map: HashMap<String, UntrackedDynamicAssetType>,
}

impl DynamicAssetCache {
    /// Get the cached asset handles corresponding to the given key.
    pub fn get_asset(&self, key: &String) -> Option<&UntrackedDynamicAssetType> {
        self.key_asset_map.get(key)
    }

    /// Insert an asset into the cache. The asset does not have to be loaded.
    ///
    /// Optional previous asset for the given key is returned
    pub fn insert_asset(
        &mut self,
        key: String,
        asset: UntrackedDynamicAssetType,
    ) -> Option<UntrackedDynamicAssetType> {
        self.key_asset_map.insert(key, asset)
    }
}

/// Any type implementing this trait can be assigned to asset keys as part of a dynamic
/// asset collection.
pub trait DynamicAsset: Debug + Send + Sync {
    /// Return handles to all required asset paths
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped>;

    /// Return the handle(s) defining this asset
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error>;

    /// Asset keys of other dynamic assets that this asset depends on
    ///
    /// All keys returned here will be loaded when [`Self::build`] is called.
    fn dependencies(&self) -> Vec<String> {
        return vec![];
    }
}

/// Resource to dynamically resolve keys to assets.
///
/// This resource is set by a [`LoadingState`](crate::loading_state::LoadingState) and is read when entering the corresponding Bevy [`State`](State).
/// If you want to manage your dynamic assets manually, they should be configured in a previous [`State`](State).
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

    /// Iterate over all the known key→asset mappings
    pub fn iter_assets(&self) -> impl Iterator<Item = (&str, &dyn DynamicAsset)> {
        self.key_asset_map
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_ref()))
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
    files: HashMap<State, HashMap<TypeId, Vec<String>>>,
    _marker: PhantomData<State>,
}

impl<State: States> DynamicAssetCollections<State> {
    /// Register a file containing dynamic asset definitions to be loaded and applied to the given loading state
    ///
    /// The file will be read every time the loading state is entered
    pub fn register_file<C: DynamicAssetCollection + Asset>(
        &mut self,
        loading_state: State,
        file: &str,
    ) {
        let mut dynamic_collections_for_state =
            self.files.remove(&loading_state).unwrap_or_default();
        let mut dynamic_files = dynamic_collections_for_state
            .remove(&TypeId::of::<C>())
            .unwrap_or_default();
        dynamic_files.push(file.to_owned());
        dynamic_collections_for_state.insert(TypeId::of::<C>(), dynamic_files);
        self.files
            .insert(loading_state, dynamic_collections_for_state);
    }

    /// Get all currently registered files to be loaded for the given loading state and dynamic asset collection type.
    pub fn get_files<C: DynamicAssetCollection + Asset>(
        &self,
        loading_state: &State,
    ) -> Option<&Vec<String>> {
        let files = self
            .files
            .get(loading_state)
            .expect("Failed to get list of dynamic asset collections for current loading state");
        files.get(&TypeId::of::<C>())
    }
}

impl<State: States> Default for DynamicAssetCollections<State> {
    fn default() -> Self {
        DynamicAssetCollections {
            files: HashMap::default(),
            _marker: PhantomData,
        }
    }
}
