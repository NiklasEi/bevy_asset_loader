use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example shows how to load an asset collection with dynamic assets defined in a `.ron` file.
///
/// The assets loaded in this example are defined in `assets/dynamic_asset.assets.ron`
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "dynamic_asset.assets.ron",
                )
                .load_collection::<ImageAssets>()
                .load_collection::<AudioAssets>(),
        )
        .add_systems(
            OnEnter(MyStates::Next),
            (spawn_player_and_tree, play_background_audio),
        )
        .add_systems(
            Update,
            animate_sprite_system.run_if(in_state(MyStates::Next)),
        )
        .run();
}

// The keys used here are defined in `assets/dynamic_asset_ron.assets`
#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(key = "layout.player_sheet")]
    player_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "image.player_sheet")]
    player: Handle<Image>,
    #[asset(key = "image.tree")]
    tree: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(key = "sounds.background")]
    background: Handle<AudioSource>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2d);
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands.spawn((
        Sprite::from_atlas_image(
            image_assets.player.clone(),
            TextureAtlas::from(image_assets.player_layout.clone()),
        ),
        Transform::from_translation(Vec3::new(0., 150., 0.)),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player,
    ));
    commands.spawn((
        Sprite::from_image(image_assets.tree.clone()),
        Transform::from_translation(Vec3::new(50., 30., 1.)),
    ));
}

fn play_background_audio(mut commands: Commands, audio_assets: Res<AudioAssets>) {
    commands.spawn((
        AudioPlayer(audio_assets.background.clone()),
        PlaybackSettings::LOOP,
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut timer, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % 8;
            }
        }
    }
}
