use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct PathAndKey {
    #[asset(path = "test.ogg", key = "test")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct StandardMaterialAndKey {
    #[asset(standard_material)]
    #[asset(key = "test")]
    test: Handle<StandardMaterial>,
}

#[derive(AssetCollection)]
struct AssetCollectionAndKey {
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 100., columns = 1, rows = 1))]
    #[asset(key = "test")]
    test: Handle<TextureAtlas>,
}

// This combination is allowed for optional dynamic assets
#[derive(AssetCollection)]
struct OptionalDynamic {
    #[asset(key = "test", optional)]
    test: Option<Handle<TextureAtlas>>,
}

// dynamic folder
#[derive(AssetCollection)]
struct FolderAndKey {
    #[asset(collection, key = "test")]
    test: Vec<HandleUntyped>,
}
