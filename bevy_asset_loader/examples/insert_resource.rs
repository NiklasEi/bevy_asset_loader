use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

/// This example demonstrates how you can use [AssetLoader::init_resource] to initialize
/// assets implementing [FromWorld] after your collections are inserted into the ECS.
///
/// In this showcase we load two textures in an [AssetCollection] and then combine
/// them by adding up their image data.
fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading, MyStates::Next)
        .with_collection::<TextureAssets>()
        .init_resource::<CombinedTexture>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(draw.system()))
        .run();
}

#[derive(AssetCollection)]
struct TextureAssets {
    #[asset(path = "textures/player.png")]
    player: Handle<Texture>,
    #[asset(path = "textures/tree.png")]
    tree: Handle<Texture>,
}

struct CombinedTexture {
    combined: Handle<Texture>,
}

impl FromWorld for CombinedTexture {
    fn from_world(world: &mut World) -> Self {
        let cell = world.cell();
        let mut textures = cell
            .get_resource_mut::<Assets<Texture>>()
            .expect("Failed to get Assets<Texture>");
        let texture_assets = cell
            .get_resource::<TextureAssets>()
            .expect("Failed to get SmallPlayerAsset");
        let player_texture = textures.get(texture_assets.player.clone()).unwrap();
        let tree_texture = textures.get(texture_assets.tree.clone()).unwrap();
        let mut combined = player_texture.clone();
        combined.data = combined
            .data
            .drain(..)
            .enumerate()
            .map(|(index, player_value)| {
                player_value
                    .checked_add(tree_texture.data[index].clone())
                    .unwrap_or(u8::MAX)
            })
            .collect();
        CombinedTexture {
            combined: textures.add(combined),
        }
    }
}

fn draw(
    mut commands: Commands,
    combined_texture: Res<CombinedTexture>,
    texture_assets: Res<TextureAssets>,
    mut material: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        material: material.add(texture_assets.player.clone().into()),
        transform: Transform::from_translation(Vec3::new(-150., 0., 1.)),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        material: material.add(combined_texture.combined.clone().into()),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        material: material.add(texture_assets.tree.clone().into()),
        transform: Transform::from_translation(Vec3::new(150., 0., 1.)),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
