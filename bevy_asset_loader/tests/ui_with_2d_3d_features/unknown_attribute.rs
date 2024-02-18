use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct Test {
    #[asset(what_is_this = "I don't know this")]
    #[asset(path = "test.ogg")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct Test2 {
    #[asset(paths = "test.ogg")]
    test: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct Test3 {
    #[asset(texture_atlas_layout(what_is_this = 2))]
    test: Handle<TextureAtlasLayout>
}
