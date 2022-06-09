use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how you can use [`LoadingState::init_resource`] to initialize
/// assets implementing [`FromWorld`] after your collections are inserted into the ECS.
///
/// In this showcase we load two images in an [`AssetCollection`] and then combine
/// them by adding up their pixel data.
fn main() {
    App::new()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_collection::<ImageAssets>()
                .init_resource::<CombinedImage>(),
        )
        .add_state(MyStates::AssetLoading)
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(draw))
        .run();
}

#[derive(AssetCollection)]
struct ImageAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
}

struct CombinedImage {
    combined: Handle<Image>,
}

impl FromWorld for CombinedImage {
    fn from_world(world: &mut World) -> Self {
        let cell = world.cell();
        let mut images = cell
            .get_resource_mut::<Assets<Image>>()
            .expect("Failed to get Assets<Image>");
        let image_assets = cell
            .get_resource::<ImageAssets>()
            .expect("Failed to get ImageAssets");
        let player_image = images.get(image_assets.player.clone()).unwrap();
        let tree_image = images.get(image_assets.tree.clone()).unwrap();
        let mut combined = player_image.clone();
        combined.data = combined
            .data
            .drain(..)
            .enumerate()
            .map(|(index, player_value)| {
                player_value
                    .checked_add(tree_image.data[index])
                    .unwrap_or(u8::MAX)
            })
            .collect();
        CombinedImage {
            combined: images.add(combined),
        }
    }
}

fn draw(
    mut commands: Commands,
    combined_texture: Res<CombinedImage>,
    image_assets: Res<ImageAssets>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        texture: image_assets.player.clone(),
        transform: Transform::from_translation(Vec3::new(-150., 0., 1.)),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        texture: combined_texture.combined.clone(),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(150., 0., 1.)),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
