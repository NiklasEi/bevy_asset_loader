use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

/// This example shows how to load an asset collection with dynamic assets defined in a `.ron` file.
///
/// The assets loaded in this example are defined in `assets/dynamic_asset_ron.assets`
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_asset_collection_file("dynamic_asset_ron.assets")
        .with_collection::<ImageAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .insert_resource(Msaa { samples: 1 })
        .add_system_set(
            SystemSet::on_enter(MyStates::Next)
                .with_system(spawn_player_and_tree.system())
                .with_system(play_background_audio.system()),
        )
        .run();
}

#[derive(AssetCollection)]
struct ImageAssets {
    #[asset(key = "image.player")]
    player: Handle<Image>,
    #[asset(key = "image.tree")]
    tree: Handle<Image>,
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(key = "sounds.background")]
    background: Handle<AudioSource>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands
        .spawn_bundle(SpriteBundle {
            texture: image_assets.player.clone(),
            transform,
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

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}

#[derive(Component)]
struct Player;
