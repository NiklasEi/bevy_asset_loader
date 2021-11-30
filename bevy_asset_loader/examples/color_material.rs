use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

/// This example demonstrates how to load a color material from a .png file
///
/// Requires the feature 'sprite' (part of default features)
fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<MyAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(spawn_player.system()))
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(color_material)]
    #[asset(path = "textures/player.png")]
    player: Handle<ColorMaterial>,
}

fn spawn_player(mut commands: Commands, texture_assets: Res<MyAssets>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        material: texture_assets.player.clone(),
        transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
