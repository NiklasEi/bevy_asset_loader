use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct PathAndKey {
    #[asset(path = "test.ogg", key = "test")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct StandardMaterialAndKey {
    #[asset(standard_material)]
    #[asset(key = "test")]
    test: Handle<StandardMaterial>,
}

#[derive(AssetCollection, Resource)]
struct AssetCollectionAndKey {
    #[asset(texture_atlas_layout(tile_size_x = 100., tile_size_y = 100., columns = 1, rows = 1))]
    #[asset(key = "test")]
    test: Handle<TextureAtlasLayout>,
}

// This combination is allowed for optional dynamic assets
#[derive(AssetCollection, Resource)]
struct OptionalDynamic {
    #[asset(key = "test", optional)]
    test: Option<Handle<TextureAtlasLayout>>,
}

// dynamic folder
#[derive(AssetCollection, Resource)]
struct FolderAndKey {
    #[asset(collection, key = "test")]
    test: Vec<UntypedHandle>,
}
