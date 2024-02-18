use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct Test {
    #[asset(texture_atlas_layout(columns = 2))]
    test: Handle<TextureAtlasLayout>
}
