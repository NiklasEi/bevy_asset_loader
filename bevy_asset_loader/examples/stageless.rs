use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_loopless::prelude::*;

const PLAYER_SPEED: f32 = 5.;

/// This example shows how to use `iyes_loopless` with `bevy_asset_loader`
fn main() {
    let mut app = App::new();
    app.add_loopless_state(MyStates::AssetLoading);
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<ImageAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    app.insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_enter_system(MyStates::Next, spawn_player_and_tree)
        .add_enter_system(MyStates::Next, play_background_audio)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(MyStates::Next)
                .with_system(move_player)
                .into(),
        )
        .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct ImageAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
    #[asset(path = "images", collection)]
    _images: Vec<HandleUntyped>,
}

#[derive(Component)]
struct Player;

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            texture: image_assets.player.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..Default::default()
        })
        .insert(Player);
    commands.spawn_bundle(SpriteBundle {
        texture: image_assets.tree.clone(),
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
    let mut transform = player.single_mut();
    transform.translation += movement;
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
