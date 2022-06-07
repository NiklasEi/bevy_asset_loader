use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
enum Test {
    #[asset(path = "test.ogg")]
    Asset(Handle<AudioSource>)
}
