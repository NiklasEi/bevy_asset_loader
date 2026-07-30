use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates the `settings` attribute, which lets you pass
/// arbitrary asset loader settings when loading an asset.
///
/// The `settings` value is any expression that evaluates to a closure matching
/// the loader's settings type (e.g., `ImageLoaderSettings` for images). Fields
/// loaded this way are loaded through `AssetServer::load_with_settings`.
///
/// The three fields below each use a different image to show three different
/// samplers:
///
///  - left sprite: `nearest` filtering (crisp pixels when scaled up)
///  - middle sprite: `linear` filtering (smooth/blurry when scaled up)
///  - right quad: `nearest` filtering with a `repeat` address mode, drawn on a
///    mesh whose UVs go beyond `[0, 1]` so the tiling is visible
///
/// Note: `AssetServer` deduplicates loads by path, so loading the *same* path
/// twice with different settings returns the same handle and ignores the second
/// settings (it even logs a warning). That is why each field below loads a
/// different image rather than the same one three times.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<ImageAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), display_images)
        .run();
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    // Nearest filtering configured through an inline `settings` closure.
    #[asset(path = "images/pixel_tree.png", settings = |settings: &mut ImageLoaderSettings| {
        settings.sampler = ImageSampler::nearest();
    })]
    tree_nearest: Handle<Image>,

    // A different image, configured with linear filtering.
    #[asset(path = "images/tree.png", settings = |settings: &mut ImageLoaderSettings| {
        settings.sampler = ImageSampler::linear();
    })]
    tree_linear: Handle<Image>,

    // A different image, configured with a full sampler descriptor: nearest
    // filtering plus a `repeat` address mode so the texture tiles.
    #[asset(path = "images/player.png", settings = |settings: &mut ImageLoaderSettings| {
        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Nearest,
            min_filter: ImageFilterMode::Nearest,
            mipmap_filter: ImageFilterMode::Nearest,
            ..default()
        });
    })]
    player_nearest_repeat: Handle<Image>,
}

fn display_images(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Left: nearest sampler (crisp pixels when scaled up).
    commands.spawn((
        Sprite {
            image: image_assets.tree_nearest.clone(),
            ..default()
        },
        Transform::from_translation(Vec3::new(-360., 40., 0.)).with_scale(Vec3::splat(8.)),
    ));
    commands.spawn((
        Text2d::new("settings: nearest"),
        Transform::from_translation(Vec3::new(-360., -80., 0.)),
    ));

    // Middle: linear sampler (smooth/blurry when scaled up).
    commands.spawn((
        Sprite {
            image: image_assets.tree_linear.clone(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0., 40., 0.)).with_scale(Vec3::splat(2.)),
    ));
    commands.spawn((
        Text2d::new("settings: linear"),
        Transform::from_translation(Vec3::new(0., -80., 0.)),
    ));

    // Right: nearest sampler with a `repeat` address mode. The `repeat` wrap is
    // only visible when UVs exceed `[0, 1]`, so we scale the material's UVs.
    // With `repeat` the tree tiles 3x3; with `clamp` the edge pixels would
    // stretch.
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(128., 128.))),
        MeshMaterial2d(materials.add(ColorMaterial {
            texture: Some(image_assets.player_nearest_repeat.clone()),
            uv_transform: Affine2::from_scale(Vec2::new(3., 3.)),
            ..default()
        })),
        Transform::from_translation(Vec3::new(360., 40., 0.)),
    ));
    commands.spawn((
        Text2d::new("settings: nearest + repeat"),
        Transform::from_translation(Vec3::new(360., -80., 0.)),
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
