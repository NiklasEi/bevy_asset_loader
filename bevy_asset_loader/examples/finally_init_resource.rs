use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how you can use [`LoadingState::finally_init_resource`] to initialize
/// assets implementing [`FromWorld`] after your collections are inserted into the ECS.
/// The same is possible with [`LoadingStateConfig::finally_init_resource`] from anywhere in your Bevy application
///
/// In this showcase we load two images in an [`AssetCollection`] and then combine
/// them by adding up their pixel data.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<ImageAssets>()
                .finally_init_resource::<CombinedImage>(),
        )
        .add_systems(OnEnter(MyStates::Next), draw)
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
        let mut system_state = SystemState::<(ResMut<Assets<Image>>, Res<ImageAssets>)>::new(world);
        let (mut images, image_assets) = system_state.get_mut(world);
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
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(image_assets.player.clone()),
        Transform::from_translation(Vec3::new(-150., 0., 1.)),
    ));
    commands.spawn(Sprite::from_image(combined_texture.combined.clone()));
    commands.spawn((
        Sprite::from_image(image_assets.tree.clone()),
        Transform::from_translation(Vec3::new(150., 0., 1.)),
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
