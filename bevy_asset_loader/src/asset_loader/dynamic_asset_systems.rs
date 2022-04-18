#[cfg(feature = "dynamic_assets")]
use crate::asset_loader::dynamic_asset::DynamicAssetCollection;
#[cfg(feature = "dynamic_assets")]
use crate::asset_loader::{AssetLoaderConfiguration, DynamicAssets, LoadingState};
#[cfg(feature = "dynamic_assets")]
use bevy::asset::LoadState;
#[cfg(feature = "dynamic_assets")]
use bevy::ecs::schedule::StateData;
#[cfg(feature = "dynamic_assets")]
use bevy::ecs::system::SystemState;
#[cfg(feature = "dynamic_assets")]
use bevy::prelude::{AssetServer, Assets, Res, ResMut, State, World};

#[cfg(feature = "dynamic_assets")]
pub(crate) fn load_dynamic_asset_collections<S: StateData>(world: &mut World) {
    let mut system_state: SystemState<(
        ResMut<AssetLoaderConfiguration<S>>,
        ResMut<State<LoadingState>>,
        Res<AssetServer>,
        Res<State<S>>,
    )> = SystemState::new(world);
    let (mut asset_loader_config, mut loading_state, asset_server, state) =
        system_state.get_mut(world);

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
    let mut system_state: SystemState<(
        Res<AssetServer>,
        ResMut<AssetLoaderConfiguration<S>>,
        ResMut<State<LoadingState>>,
        ResMut<Assets<DynamicAssetCollection>>,
        ResMut<DynamicAssets>,
    )> = SystemState::new(world);
    let (
        asset_server,
        mut asset_loader_configuration,
        mut loading_state,
        mut dynamic_asset_collections,
        mut asset_keys,
    ) = system_state.get_mut(world);

    let collections_load_state = asset_server.get_group_load_state(
        asset_loader_configuration
            .asset_collection_handles
            .iter()
            .map(|handle| handle.id),
    );
    if collections_load_state == LoadState::Loaded {
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
