use bevy_asset_loader::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection)]
struct Test {
    #[asset(texture_atlas(columns = 2))]
    #[asset(path = "test.ogg")]
    test: Handle<AudioSource>
}
