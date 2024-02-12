use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

/// Todo: This should fail to compile!!
#[derive(AssetCollection, Resource)]
struct Test {
    #[asset(texture_atlas(columns = 2))]
    test: Handle<TextureAtlasLayout>
}
