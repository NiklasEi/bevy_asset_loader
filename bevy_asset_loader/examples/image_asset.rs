use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how you can set a different sampler for an [`Image`].
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<ImageAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), (draw, assert))
        .run();
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler(filter = linear)))]
    tree_linear: Handle<Image>,

    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler(filter = nearest)))]
    tree_nearest: Handle<Image>,

    #[asset(path = "images/array_texture.png")]
    #[asset(image(array_texture_layers = 4))]
    array_texture: Handle<Image>,
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

fn assert(images: Res<ImageAssets>, image_assets: Res<Assets<Image>>) {
    let array_texture = image_assets.get(&images.array_texture).unwrap();
    assert_eq!(
        array_texture.texture_descriptor.array_layer_count(),
        4,
        "The image should have been reinterpreted as array texture with 4 layers"
    );
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
