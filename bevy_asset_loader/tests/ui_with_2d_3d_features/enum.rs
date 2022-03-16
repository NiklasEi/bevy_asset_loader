use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
enum Test {
    #[asset(path = "test.ogg")]
    Asset(Handle<AudioSource>)
}
