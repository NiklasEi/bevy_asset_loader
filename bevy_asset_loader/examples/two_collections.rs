use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

const PLAYER_SPEED: f32 = 5.;

/// This example shows how to load multiple asset collections with one [AssetLoader]
fn main() {
    let mut app = App::build();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<TextureAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_system_set(
            SystemSet::on_enter(MyStates::Next)
                .with_system(spawn_player_and_tree.system())
                .with_system(play_background_audio.system()),
        )
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(move_player.system()))
        .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct TextureAssets {
    #[asset(path = "textures/player.png")]
    player: Handle<Texture>,
    #[asset(path = "textures/tree.png")]
    tree: Handle<Texture>,
}

struct Player;

fn spawn_player_and_tree(
    mut commands: Commands,
    texture_assets: Res<TextureAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_assets.player.clone().into()),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..Default::default()
        })
        .insert(Player);
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(texture_assets.tree.clone().into()),
        transform: Transform::from_translation(Vec3::new(50., 30., 1.)),
        ..Default::default()
    });
}

fn play_background_audio(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.play(audio_assets.background.clone());
}

fn move_player(input: Res<Input<KeyCode>>, mut player: Query<&mut Transform, With<Player>>) {
    let mut movement = Vec3::new(0., 0., 0.);
    if input.pressed(KeyCode::W) {
        movement.y += 1.;
    }
    if input.pressed(KeyCode::S) {
        movement.y -= 1.;
    }
    if input.pressed(KeyCode::A) {
        movement.x -= 1.;
    }
    if input.pressed(KeyCode::D) {
        movement.x += 1.;
    }
    if movement == Vec3::ZERO {
        return;
    }
    movement = movement.normalize() * PLAYER_SPEED;
    if let Ok(mut transform) = player.single_mut() {
        transform.translation += movement;
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
