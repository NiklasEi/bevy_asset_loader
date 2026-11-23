use std::any::TypeId;
use std::fmt::Debug;

use bevy_asset::{Asset, AssetServer, UntypedHandle};
use bevy_ecs::{resource::Resource, world::World};
use bevy_platform::collections::HashMap;
use bevy_state::state::FreelyMutableState;
use std::marker::PhantomData;

/// Different typed that can generate the asset field value of a dynamic asset
#[derive(Debug)]
pub enum DynamicAssetType {
    /// Dynamic asset that is defined by a single handle
    Single(UntypedHandle),
    /// Dynamic asset that is defined by multiple handles
    Collection(Vec<UntypedHandle>),
    /// Dynamic asset that is defined by a key for each of its values
    Map(HashMap<String, DynamicAssetType>),
}

impl DynamicAssetType {
    /// All handles of this dynamic asset, including the handles in nested collections and maps
    pub fn into_handles(self) -> Vec<UntypedHandle> {
        match self {
            DynamicAssetType::Single(handle) => vec![handle],
            DynamicAssetType::Collection(handles) => handles,
            DynamicAssetType::Map(map) => map
                .into_values()
                .flat_map(DynamicAssetType::into_handles)
                .collect(),
        }
    }
}

/// Any type implementing this trait can be assigned to asset keys as part of a dynamic
/// asset collection.
pub trait DynamicAsset: Debug + Send + Sync {
    /// Return handles to all required asset paths
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle>;

    /// Return the handle(s) defining this asset
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error>;

    /// Like [`load`](Self::load), but for an asset collection field that holds handles of the
    /// asset type `asset_type`.
    ///
    /// Implement this when your dynamic asset loads files by path. Loading them as `asset_type`
    /// picks the loader for that type, even if another loader is registered for the same file
    /// extension. The default implementation ignores the type and calls [`load`](Self::load).
    ///
    /// If you implement this, also implement [`build_as`](Self::build_as).
    fn load_as(&self, world: &mut World, asset_type: TypeId) -> Vec<UntypedHandle> {
        let _ = asset_type;
        let asset_server = world.resource::<AssetServer>().clone();
        self.load(&asset_server)
    }

    /// Like [`build`](Self::build), but for assets loaded with [`load_as`](Self::load_as).
    ///
    /// The default implementation ignores the type and calls [`build`](Self::build).
    fn build_as(
        &self,
        world: &mut World,
        asset_type: TypeId,
    ) -> Result<DynamicAssetType, anyhow::Error> {
        let _ = asset_type;
        self.build(world)
    }

    /// Get an entry of this dynamic asset, like the value for a key of a map or the element
    /// at an index of an array.
    ///
    /// Asset collections address entries with keys like `"icons[play]"` or `"layouts[0]"`.
    fn entry(&self, key: &str) -> Option<&dyn DynamicAsset> {
        let _ = key;
        None
    }
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
        self.resolve(key).ok()
    }

    /// Like [`get_asset`](Self::get_asset), but describes which part of the key could not be found.
    pub(crate) fn resolve(&self, key: &str) -> Result<&dyn DynamicAsset, MissingDynamicAsset> {
        if let Some(asset) = self.key_asset_map.get(key) {
            return Ok(asset.as_ref());
        }
        let Some((base, entries)) = split_entries(key) else {
            return Err(MissingDynamicAsset::Key(key.to_owned()));
        };
        let mut asset = self
            .key_asset_map
            .get(base)
            .ok_or_else(|| MissingDynamicAsset::Key(base.to_owned()))?
            .as_ref();
        let mut parent = base.to_owned();
        for entry in entries {
            asset = asset
                .entry(entry)
                .ok_or_else(|| MissingDynamicAsset::Entry {
                    parent: parent.clone(),
                    entry: entry.to_owned(),
                })?;
            parent = format!("{parent}[{entry}]");
        }
        Ok(asset)
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

/// Split a key like `"icons[play]"` into the base key and the addressed entries.
///
/// Returns `None` if the key does not address any entries.
fn split_entries(key: &str) -> Option<(&str, Vec<&str>)> {
    let (base, mut rest) = key.split_once('[')?;
    if base.is_empty() {
        return None;
    }
    let mut entries = vec![];
    loop {
        let (entry, after) = rest.split_once(']')?;
        if entry.is_empty() || entry.contains('[') {
            return None;
        }
        entries.push(entry);
        if after.is_empty() {
            return Some((base, entries));
        }
        rest = after.strip_prefix('[')?;
    }
}

/// A key that could not be resolved to a dynamic asset
#[derive(Debug, PartialEq)]
pub(crate) enum MissingDynamicAsset {
    /// There is no dynamic asset registered for the key
    Key(String),
    /// The dynamic asset at `parent` has no entry `entry`
    Entry { parent: String, entry: String },
}

/// This traits describes types that contain asset configurations and can
/// register them in the [`DynamicAssets`] resource.
pub trait DynamicAssetCollection {
    /// Register all dynamic assets inside the collection in the [`DynamicAssets`] resource.
    fn register(&self, dynamic_assets: &mut DynamicAssets);
}

/// Resource keeping track of dynamic asset collection files for different loading states
#[derive(Resource, Debug)]
pub struct DynamicAssetCollections<State: FreelyMutableState> {
    files: HashMap<State, HashMap<TypeId, Vec<String>>>,
    _marker: PhantomData<State>,
}

impl<State: FreelyMutableState> DynamicAssetCollections<State> {
    /// Register a file containing dynamic asset definitions to be loaded and applied to the given loading state
    ///
    /// The file will be read every time the loading state is entered
    pub fn register_file<C: DynamicAssetCollection + Asset>(
        &mut self,
        loading_state: State,
        file: &str,
    ) {
        self.register_file_by_type_id(loading_state, file, TypeId::of::<C>());
    }

    pub(crate) fn register_files_by_type_id(
        &mut self,
        loading_state: State,
        mut files: Vec<String>,
        type_id: TypeId,
    ) {
        let mut dynamic_collections_for_state =
            self.files.remove(&loading_state).unwrap_or_default();
        let mut dynamic_files = dynamic_collections_for_state
            .remove(&type_id)
            .unwrap_or_default();
        dynamic_files.append(&mut files);
        dynamic_collections_for_state.insert(type_id, dynamic_files);
        self.files
            .insert(loading_state, dynamic_collections_for_state);
    }

    pub(crate) fn register_file_by_type_id(
        &mut self,
        loading_state: State,
        file: &str,
        type_id: TypeId,
    ) {
        let mut dynamic_collections_for_state =
            self.files.remove(&loading_state).unwrap_or_default();
        let mut dynamic_files = dynamic_collections_for_state
            .remove(&type_id)
            .unwrap_or_default();
        dynamic_files.push(file.to_owned());
        dynamic_collections_for_state.insert(type_id, dynamic_files);
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

impl<State: FreelyMutableState> Default for DynamicAssetCollections<State> {
    fn default() -> Self {
        DynamicAssetCollections {
            files: HashMap::default(),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dynamic asset with named entries
    #[derive(Debug)]
    struct Node(Vec<(&'static str, Node)>);

    impl DynamicAsset for Node {
        fn load(&self, _: &AssetServer) -> Vec<UntypedHandle> {
            vec![]
        }

        fn build(&self, _: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
            Ok(DynamicAssetType::Collection(vec![]))
        }

        fn entry(&self, key: &str) -> Option<&dyn DynamicAsset> {
            self.0
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, node)| node as &dyn DynamicAsset)
        }
    }

    fn dynamic_assets() -> DynamicAssets {
        let mut dynamic_assets = DynamicAssets::default();
        dynamic_assets.register_asset(
            "enemies",
            Box::new(Node(vec![("boss", Node(vec![("attack", Node(vec![]))]))])),
        );
        dynamic_assets.register_asset("weird[key]", Box::new(Node(vec![])));
        dynamic_assets
    }

    fn resolves_to_leaf(key: &str) -> bool {
        dynamic_assets()
            .resolve(key)
            .is_ok_and(|asset| format!("{asset:?}") == "Node([])")
    }

    #[test]
    fn resolve_nested_entries() {
        assert!(resolves_to_leaf("enemies[boss][attack]"));
        assert!(dynamic_assets().get_asset("enemies[boss]").is_some());
        assert!(resolves_to_leaf("weird[key]"));
        assert_eq!(
            dynamic_assets()
                .resolve("enemies[boss][defend]")
                .unwrap_err(),
            MissingDynamicAsset::Entry {
                parent: "enemies[boss]".to_owned(),
                entry: "defend".to_owned()
            }
        );
        assert_eq!(
            dynamic_assets().resolve("enemy[boss]").unwrap_err(),
            MissingDynamicAsset::Key("enemy".to_owned())
        );
        assert_eq!(
            dynamic_assets().resolve("enemies[boss").unwrap_err(),
            MissingDynamicAsset::Key("enemies[boss".to_owned())
        );
    }

    #[test]
    fn split_keys_into_entries() {
        assert_eq!(split_entries("icons"), None);
        assert_eq!(split_entries("icons[play]"), Some(("icons", vec!["play"])));
        assert_eq!(
            split_entries("enemies.big[boss][0]"),
            Some(("enemies.big", vec!["boss", "0"]))
        );
        assert_eq!(split_entries("[play]"), None);
        assert_eq!(split_entries("icons[]"), None);
        assert_eq!(split_entries("icons[play"), None);
        assert_eq!(split_entries("icons[play]x"), None);
        assert_eq!(split_entries("icons[pl[ay]"), None);
    }
}
