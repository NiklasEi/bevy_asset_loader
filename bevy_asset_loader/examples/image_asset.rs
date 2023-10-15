use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how you can use [`App::init_resource_after_loading_state`] to initialize
/// assets implementing [`FromWorld`] after your collections are inserted into the ECS.
///
/// In this showcase we load two images in an [`AssetCollection`] and then combine
/// them by adding up their pixel data.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading).continue_to_state(MyStates::Next),
        )
        .add_collection_to_loading_state::<_, ImageAssets>(MyStates::AssetLoading)
        .insert_resource(Msaa::Off)
        .add_systems(OnEnter(MyStates::Next), draw)
        .run();
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/player.png")]
    #[asset(image(sampler = linear))]
    player: Handle<Image>,

    #[asset(path = "images/tree.png")]
    #[asset(image(sampler = nearest))]
    tree: Handle<Image>,
}

fn draw(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: image_assets.player.clone(),
        transform: Transform::from_translation(Vec3::new(-150., 0., 1.)),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(150., 0., 1.)),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
