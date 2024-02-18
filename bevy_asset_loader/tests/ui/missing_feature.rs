use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct Test {
    #[asset(texture_atlas_layout(tile_size_x = 100., tile_size_y = 100., columns = 1, rows = 1))]
    atlas: Handle<TextureAtlasLayout>,
    #[asset(standard_material)]
    material: Handle<StandardMaterial>,
}
