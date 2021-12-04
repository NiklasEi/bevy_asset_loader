use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct PathAndKey {
    #[asset(path = "test.ogg", key = "test")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct FolderAndKey {
    #[asset(folder = "folder", key = "test")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct ColorMaterialAndKey {
    #[asset(color_material)]
    #[asset(key = "test")]
    test: Handle<ColorMaterial>,
}

#[derive(AssetCollection)]
struct AssetCollectionAndKey {
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 100., columns = 1, rows = 1))]
    #[asset(key = "test")]
    test: Handle<TextureAtlas>,
}
