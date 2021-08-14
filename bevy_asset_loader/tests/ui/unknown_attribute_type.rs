use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct Test {
    #[asset(what_is_this)]
    #[asset(path = "test.ogg")]
    test: Handle<AudioSource>,
}
