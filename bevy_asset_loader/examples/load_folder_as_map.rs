use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how to load a folder as a map of asset path to handle
fn main() {
    App::new()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_collection::<MyAssets>(),
        )
        .add_state(MyStates::AssetLoading)
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(use_assets))
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "images", collection(mapped, typed))]
    images: HashMap<String, Handle<Image>>,
    #[asset(path = "images", collection(mapped))]
    _untyped_images: HashMap<String, HandleUntyped>,
}

fn use_assets(mut commands: Commands, my_assets: Res<MyAssets>) {
    commands.spawn(Camera2dBundle::default());
    let tree = my_assets
        .images
        .get("tree.png")
        .expect("Failed to find tree");
    commands.spawn(SpriteBundle {
        texture: tree.clone(),
        ..default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
