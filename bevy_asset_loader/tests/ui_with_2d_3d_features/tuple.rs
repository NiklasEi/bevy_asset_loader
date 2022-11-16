use bevy_asset_loader::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct Test(usize);
