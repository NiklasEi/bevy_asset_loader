use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {}

#[derive(AssetCollection, Resource)]
struct Test(usize);
