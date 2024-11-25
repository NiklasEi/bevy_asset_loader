use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how to configure an existing loading state from a separate plugin
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MainPlugin, PlayerAndMusicPlugin))
        .run();
}

struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MyStates>()
            // General loading state setup goes here, but if you like, you can already add all
            // the configuration at this point, too. In this example we will configure the loading state
            // later in PlayerAndMusicPlugin.
            .add_loading_state(
                LoadingState::new(MyStates::AssetLoading).continue_to_state(MyStates::Next),
            )
            .add_systems(
                OnEnter(MyStates::Next),
                (spawn_player, play_background_audio),
            );
    }
}

struct PlayerAndMusicPlugin;

impl Plugin for PlayerAndMusicPlugin {
    fn build(&self, app: &mut App) {
        app
            // We can add all kinds of things to the loading state here. This method can be called
            // from any plugin any number of times.
            .configure_loading_state(
                LoadingStateConfig::new(MyStates::AssetLoading)
                    .load_collection::<AudioAssets>()
                    .load_collection::<ImageAssets>()
                    .init_resource::<ExampleResource>(),
            );
    }
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
}

#[derive(Resource)]
struct ExampleResource {
    _resource: &'static str,
}

impl FromWorld for ExampleResource {
    fn from_world(_world: &mut World) -> Self {
        ExampleResource {
            _resource: "You could use the ECS World here!",
        }
    }
}

fn spawn_player(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(image_assets.player.clone()),
        Transform::from_translation(Vec3::new(0., 0., 1.)),
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
