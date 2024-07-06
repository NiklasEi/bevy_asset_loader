use bevy_asset_loader::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new().init_collection::<Test>();
}

#[derive(Resource)]
struct Test {
    material: Handle<StandardMaterial>,
}
