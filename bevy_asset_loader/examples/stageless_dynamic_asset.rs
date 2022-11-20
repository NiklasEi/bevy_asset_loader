use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::*;

/// This example shows how to load an asset collection with dynamic assets defined in a `.ron` file.
///
/// The assets loaded in this example are defined in `assets/dynamic_asset.assets`
fn main() {
    App::new()
        .add_loopless_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<ImageAssets>()
                .with_collection::<AudioAssets>(),
        )
        .insert_resource(Msaa { samples: 1 })
        .add_enter_system(MyStates::Next, spawn_player_and_tree)
        .add_enter_system(MyStates::Next, play_background_audio)
        .add_system(animate_sprite_system.run_in_state(MyStates::Next))
        .run();
}

// The keys used here are defined in `assets/dynamic_asset_ron.assets`
#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(key = "image.player")]
    player: Handle<TextureAtlas>,
    #[asset(key = "image.tree")]
    tree: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(key = "sounds.background")]
    background: Handle<AudioSource>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2dBundle::default());
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform {
                translation: Vec3::new(0., 150., 0.),
                ..Default::default()
            },
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: image_assets.player.clone(),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .insert(Player);
    commands.spawn(SpriteBundle {
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

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite)>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index = (sprite.index + 1) % 8;
        }
    }
}
