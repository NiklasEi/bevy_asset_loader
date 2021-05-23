use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

fn main() {
    let mut app = App::build();
    AssetLoader::new(MyStates::AssetLoading, MyStates::Next)
        .with_collection::<TextureAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_system_set(
            SystemSet::on_enter(MyStates::Next).with_system(spawn_player_and_tree.system()),
        )
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(move_player.system()))
        .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "walking.ogg")]
    walking: Handle<AudioSource>,
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
        transform: Transform::from_translation(Vec3::new(5., 3., 1.)),
        ..Default::default()
    });
}

fn move_player(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut player: Query<&mut Transform, With<Player>>,
    audio_assets: Res<AudioAssets>,
    audio: Res<Audio>,
) {
    let mut movement = Vec2::new(0., 0.);
    for event in keyboard_input_events.iter() {
        if let Some(key) = event.key_code {
            match key {
                KeyCode::W => movement.y += 5.,
                KeyCode::S => movement.y -= 5.,
                KeyCode::A => movement.x -= 5.,
                KeyCode::D => movement.x += 5.,
                _ => (),
            }
        }
    }
    if movement != Vec2::ZERO {
        audio.play(audio_assets.walking.clone());
    }
    if let Ok(mut transform) = player.single_mut() {
        transform.translation += Vec3::new(movement.x.clone(), movement.y.clone(), 0.)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
