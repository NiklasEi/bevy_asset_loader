use crate::dynamic_asset::DynamicAssets;
use bevy_app::App;
use bevy_asset::UntypedHandle;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;

pub use bevy_asset_loader_derive::AssetCollection;

/// Trait to mark a struct as a collection of assets
///
/// Derive is supported for structs with named fields.
/// ```edition2021
/// # use bevy_asset_loader::prelude::*;
/// # use bevy::prelude::*;
/// #[derive(AssetCollection, Resource)]
/// struct MyAssets {
///     #[asset(path = "player.png")]
///     player: Handle<Image>,
///     #[asset(path = "tree.png")]
///     tree: Handle<Image>
/// }
/// ```
pub trait AssetCollection: Resource {
    /// Create a new asset collection from the [`AssetServer`](::bevy::asset::AssetServer)
    fn create(world: &mut World) -> Self;
    /// Start loading all the assets in the collection
    fn load(world: &mut World) -> Vec<UntypedHandle>;
}

/// Extension trait for [`App`] enabling initialisation of [asset collections](crate::asset_collection::AssetCollection)
pub trait AssetCollectionApp {
    /// Initialise an [`AssetCollection`]
    ///
    /// This function does not give any guaranties about the loading status of the asset handles.
    /// If you want to use a loading state, you do not need this function! Instead use an [`LoadingState`](crate::loading_state::LoadingState)
    /// and add collections to it to be prepared during the loading state.
    fn init_collection<A: AssetCollection>(&mut self) -> &mut Self;
}

impl AssetCollectionApp for App {
    fn init_collection<Collection>(&mut self) -> &mut Self
    where
        Collection: AssetCollection,
    {
        if !self.world().contains_resource::<Collection>() {
            // This resource is required for loading a collection
            // Since bevy_asset_loader does not have a "real" Plugin,
            // we need to make sure the resource exists here
            self.init_resource::<DynamicAssets>();
            // make sure the assets start to load
            let _ = Collection::load(self.world_mut());
            let resource = Collection::create(self.world_mut());
            self.insert_resource(resource);
        }
        self
    }
}

/// Extension trait for [`World`] enabling initialisation of [asset collections](AssetCollection)
pub trait AssetCollectionWorld {
    /// Initialise an [`AssetCollection`]
    ///
    /// This function does not give any guaranties about the loading status of the asset handles.
    /// If you want such guaranties, use a [`LoadingState`](crate::loading_state::LoadingState).
    fn init_collection<A: AssetCollection>(&mut self);
}

impl AssetCollectionWorld for World {
    fn init_collection<A: AssetCollection>(&mut self) {
        if self.get_resource::<A>().is_none() {
            // This resource is required for loading a collection
            // Since bevy_asset_loader can be used without adding a plugin,
            // we need to make sure the resource exists here
            self.init_resource::<DynamicAssets>();
            let collection = A::create(self);
            self.insert_resource(collection);
        }
    }
}
