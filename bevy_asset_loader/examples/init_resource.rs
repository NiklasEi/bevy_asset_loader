use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how you can use [`App::init_resource_after_loading_state`] to initialize
/// assets implementing [`FromWorld`] after your collections are inserted into the ECS.
///
/// In this showcase we load two images in an [`AssetCollection`] and then combine
/// them by adding up their pixel data.
fn main() {
    App::new()
        .add_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading).continue_to_state(MyStates::Next),
        )
        .add_collection_to_loading_state::<_, ImageAssets>(MyStates::AssetLoading)
        .init_resource_after_loading_state::<_, CombinedImage>(MyStates::AssetLoading)
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins)
        .add_system(draw.in_schedule(OnEnter(MyStates::Next)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
}

#[derive(Resource)]
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
        let player_image = images.get(&image_assets.player).unwrap();
        let tree_image = images.get(&image_assets.tree).unwrap();
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
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: image_assets.player.clone(),
        transform: Transform::from_translation(Vec3::new(-150., 0., 1.)),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: combined_texture.combined.clone(),
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
