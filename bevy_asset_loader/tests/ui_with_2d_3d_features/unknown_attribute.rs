use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct Test {
    #[asset(what_is_this = "I don't know this")]
    #[asset(path = "test.ogg")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct Test2 {
    #[asset(paths = "test.ogg")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct Test3 {
    #[asset(texture_atlas(what_is_this = 2))]
    #[asset(path = "test.png")]
    test: Handle<TextureAtlas>
}
