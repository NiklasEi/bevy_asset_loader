use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct TestPath {
    #[asset(path = 1)]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct TestTextureAtlas {
    #[asset(texture_atlas(tile_size_x = 100, tile_size_y = 100., columns = 1, rows = 1))]
    #[asset(path = "asset.png")]
    test: Handle<TextureAtlas>,
}

#[derive(AssetCollection, Resource)]
struct TestTextureAtlasSecond {
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 100., columns = "5", rows = 1))]
    #[asset(path = "asset.png")]
    test: Handle<TextureAtlas>,
}
