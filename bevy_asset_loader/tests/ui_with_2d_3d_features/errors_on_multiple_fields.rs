use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct Test {
    #[asset(what_is_this)]
    #[asset(path = "test.ogg")]
    first: Handle<AudioSource>,
    #[asset(texture_atlas(tile_size_x = 100.))]
    #[asset(path = "test.png")]
    second: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 100., columns = 10., rows = 1))]
    #[asset(path = "test.png")]
    third: Handle<TextureAtlas>,
}
