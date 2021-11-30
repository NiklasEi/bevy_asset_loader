use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct Test {
    #[asset(path = "test.ogg", folder = "test")]
    test: Handle<AudioSource>,
}
