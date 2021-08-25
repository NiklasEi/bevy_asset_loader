use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct Test {
    #[asset(path = "test.ogg")]
    test: Handle<AudioSource>,
    non_default: NoDefault
}

enum NoDefault {
    One,
    Two
}
