#[cfg(feature = "dynamic_assets")]
use crate::asset_loader::dynamic_asset::DynamicAssetCollection;
#[cfg(feature = "dynamic_assets")]
use crate::asset_loader::{AssetLoaderConfiguration, DynamicAssets, LoadingState};
#[cfg(feature = "dynamic_assets")]
use bevy::asset::LoadState;
#[cfg(feature = "dynamic_assets")]
use bevy::ecs::schedule::StateData;
#[cfg(feature = "dynamic_assets")]
use bevy::prelude::{AssetServer, Assets, State, World};

#[cfg(feature = "dynamic_assets")]
pub(crate) fn load_dynamic_asset_collections<S: StateData>(world: &mut World) {
    let cell = world.cell();
    let mut asset_loader_config = cell
        .get_resource_mut::<AssetLoaderConfiguration<S>>()
        .unwrap();
    let asset_server = cell.get_resource::<AssetServer>().unwrap();
    let state = cell
        .get_resource::<State<S>>()
        .expect("Cannot get State resource");
    let mut loading_state = cell
        .get_resource_mut::<State<LoadingState>>()
        .expect("Cannot get LoadingStatePhase");

    let files = asset_loader_config.get_asset_collection_files(state.current());
    if files.is_empty() {
        loading_state
            .set(LoadingState::LoadingAssets)
            .expect("Failed to set loading state");
        return;
    }
    for file in files {
        asset_loader_config
            .asset_collection_handles
            .push(asset_server.load(&file));
    }
}

#[cfg(feature = "dynamic_assets")]
pub(crate) fn check_dynamic_asset_collections<S: StateData>(world: &mut World) {
    {
        let cell = world.cell();
        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Cannot get AssetServer resource");
        let mut asset_loader_configuration = cell
            .get_resource_mut::<AssetLoaderConfiguration<S>>()
            .expect("Cannot get AssetLoaderConfiguration");
        let mut loading_state = cell
            .get_resource_mut::<State<LoadingState>>()
            .expect("Failed to get loading state");
        let collections_load_state = asset_server.get_group_load_state(
            asset_loader_configuration
                .asset_collection_handles
                .iter()
                .map(|handle| handle.id),
        );
        if collections_load_state == LoadState::Loaded {
            let mut dynamic_asset_collections = cell
                .get_resource_mut::<Assets<DynamicAssetCollection>>()
                .expect("Cannot get AssetServer resource");

            let mut asset_keys = cell.get_resource_mut::<DynamicAssets>().unwrap();
            for collection in asset_loader_configuration
                .asset_collection_handles
                .drain(..)
            {
                let collection = dynamic_asset_collections.remove(collection).unwrap();
                asset_keys.register_dynamic_collection(collection);
            }
            loading_state
                .set(LoadingState::LoadingAssets)
                .expect("Failed to set loading state");
        }
    }
}
