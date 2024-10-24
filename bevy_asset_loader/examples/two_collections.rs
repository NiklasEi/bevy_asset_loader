use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

const PLAYER_SPEED: f32 = 5.;

/// This example shows how to load multiple asset collections in one [`LoadingState`]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<ImageAssets>()
                .load_collection::<AudioAssets>(),
        )
        .add_systems(
            OnEnter(MyStates::Next),
            (spawn_player_and_tree, play_background_audio),
        )
        .add_systems(Update, move_player.run_if(in_state(MyStates::Next)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
    #[asset(path = "images", collection)]
    _images: Vec<UntypedHandle>,
}

#[derive(Component)]
struct Player;

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(SpriteBundle {
            texture: image_assets.player.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..Default::default()
        })
        .insert(Player);
    commands.spawn(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(50., 30., 1.)),
        ..Default::default()
    });
}

fn play_background_audio(mut commands: Commands, audio_assets: Res<AudioAssets>) {
    commands.spawn((
        AudioPlayer(audio_assets.background.clone()),
        PlaybackSettings::LOOP,
    ));
}

fn move_player(input: Res<ButtonInput<KeyCode>>, mut player: Query<&mut Transform, With<Player>>) {
    let mut movement = Vec3::new(0., 0., 0.);
    if input.pressed(KeyCode::KeyW) {
        movement.y += 1.;
    }
    if input.pressed(KeyCode::KeyS) {
        movement.y -= 1.;
    }
    if input.pressed(KeyCode::KeyA) {
        movement.x -= 1.;
    }
    if input.pressed(KeyCode::KeyD) {
        movement.x += 1.;
    }
    if movement == Vec3::ZERO {
        return;
    }
    movement = movement.normalize() * PLAYER_SPEED;
    let mut transform = player.single_mut();
    transform.translation += movement;
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
