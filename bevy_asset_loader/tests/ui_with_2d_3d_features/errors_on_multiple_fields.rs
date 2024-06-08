use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct Test {
    #[asset(what_is_this)]
    #[asset(path = "test.ogg")]
    first: Handle<AudioSource>,
    #[asset(texture_atlas_layout(tile_size_x = 100))]
    second: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 100, tile_size_y = 100, columns = 10., rows = 1))]
    third: Handle<TextureAtlasLayout>,
}
