use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct Test {
    #[asset(texture_atlas(columns = 2))]
    #[asset(path = "test.png")]
    test: Handle<TextureAtlas>
}
