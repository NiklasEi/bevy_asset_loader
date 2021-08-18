use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct Test {
    #[asset(what_is_this)]
    #[asset(path = "test.ogg")]
    first: Handle<AudioSource>,
    #[asset(texture_atlas(cell_width=100.))]
    #[asset(path = "test.png")]
    second: Handle<TextureAtlas>,
    #[asset(texture_atlas(cell_width=100., cell_height=100., columns = 10., rows = 1))]
    #[asset(path = "test.png")]
    third: Handle<TextureAtlas>,
}
