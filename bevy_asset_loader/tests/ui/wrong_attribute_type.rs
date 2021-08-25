use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct TestPath {
    #[asset(path = 1)]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct TestTextureAtlas {
    #[asset(texture_atlas(cell_height = 100, cell_width=100., columns = 1, rows = 1))]
    #[asset(path = "asset.png")]
    test: Handle<TextureAtlas>,
}

#[derive(AssetCollection)]
struct TestTextureAtlasSecond {
    #[asset(texture_atlas(cell_height = 100., cell_width=100., columns = "5", rows = 1))]
    #[asset(path = "asset.png")]
    test: Handle<TextureAtlas>,
}