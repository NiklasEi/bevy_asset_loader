use bevy::utils::HashMap;
use std::any::TypeId;
use std::fmt::Debug;

use bevy::asset::{Asset, AssetServer, UntypedHandle};
use bevy::ecs::schedule::States;
use bevy::ecs::system::Resource;
use bevy::ecs::world::World;
use std::marker::PhantomData;

/// Different typed that can generate the asset field value of a dynamic asset
pub enum DynamicAssetType {
    /// Dynamic asset that is defined by a single handle
    Single(UntypedHandle),
    /// Dynamic asset that is defined by multiple handles
    Collection(Vec<UntypedHandle>),
}

/// Any type implementing this trait can be assigned to asset keys as part of a dynamic
/// asset collection.
pub trait DynamicAsset: Debug + Send + Sync {
    /// Return handles to all required asset paths
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle>;

    /// Return the handle(s) defining this asset
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error>;
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
        let files = self.files.get(loading_state)?;
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
