use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how you can set a different sampler for an [`Image`].
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading).continue_to_state(MyStates::Next),
        )
        .add_collection_to_loading_state::<_, ImageAssets>(MyStates::AssetLoading)
        .add_systems(OnEnter(MyStates::Next), draw)
        .run();
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler = linear))]
    tree_linear: Handle<Image>,

    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler = nearest))]
    tree_nearest: Handle<Image>,
}

fn draw(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far: 1000.,
            near: -1000.,
            scale: 0.25,
            ..default()
        },
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: image_assets.tree_linear.clone(),
        transform: Transform::from_translation(Vec3::new(-50., 0., 1.)),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: image_assets.tree_nearest.clone(),
        transform: Transform::from_translation(Vec3::new(50., 0., 1.)),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
