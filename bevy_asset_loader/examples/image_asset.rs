use bevy::math::Affine2;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how you can set different samplers and wrap modes for
/// an [`Image`] asset.
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

    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler(filter = nearest, wrap = repeat)))]
    tree_nearest_repeat: Handle<Image>,

    #[asset(path = "images/array_texture.png")]
    #[asset(image(array_texture_layers = 4))]
    array_texture: Handle<Image>,
}

fn draw(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            order: 1,
            ..default()
        },
        ..default()
    });
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: image_assets.tree_linear.clone(),
        transform: Transform::from_translation(Vec3::new(-50., 0., 1.)).with_scale(Vec3::splat(5.)),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: image_assets.tree_nearest.clone(),
        transform: Transform::from_translation(Vec3::new(50., 0., 1.)).with_scale(Vec3::splat(5.)),
        ..Default::default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(image_assets.tree_nearest_repeat.clone()),
            uv_transform: Affine2::from_scale(Vec2::new(2., 3.)),
            ..default()
        }),
        transform: Transform::from_xyz(1.5, 0.0, 0.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
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
