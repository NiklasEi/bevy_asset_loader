use bevy::app::AppExit;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_progress::{ProgressCounter, ProgressPlugin};

/// This example shows how to track the loading progress of your collections using `iyes_progress`
///
/// Running it will print the current progress for every frame. The five assets from
/// the two collections will be loaded rather quickly (one or two frames). The final task
/// completes after one second. At that point, `iyes_progress` will continue to the next state
/// and the app will terminate.
fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .with_collection::<TextureAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        // track progress during `MyStates::AssetLoading` and continue to `MyStates::Next` when progress is completed
        .add_plugin(ProgressPlugin::new(MyStates::AssetLoading).continue_to(MyStates::Next))
        // gracefully quit the app when `MyStates::Next` is reached
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(expect))
        .add_system_set(
            SystemSet::on_update(MyStates::AssetLoading).with_system(track_fake_long_task),
        )
        .add_system_to_stage(CoreStage::PostUpdate, print_progress)
        .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
    #[asset(path = "audio/plop.ogg")]
    plop: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct TextureAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 99., columns = 8, rows = 1))]
    #[asset(path = "images/female_adventurer_sheet.png")]
    female_adventurer: Handle<TextureAtlas>,
}

fn track_fake_long_task(time: Res<Time>, progress: Res<ProgressCounter>) {
    if time.seconds_since_startup() > 1. {
        info!("done");
        progress.manually_track(true.into());
    } else {
        progress.manually_track(false.into());
    }
}

fn expect(
    audio_assets: Res<AudioAssets>,
    texture_assets: Res<TextureAssets>,
    asset_server: Res<AssetServer>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut quit: EventWriter<AppExit>,
) {
    assert_eq!(
        asset_server.get_load_state(audio_assets.background.clone()),
        LoadState::Loaded
    );
    assert_eq!(
        asset_server.get_load_state(audio_assets.plop.clone()),
        LoadState::Loaded
    );
    let atlas = texture_atlases
        .get(texture_assets.female_adventurer.clone())
        .expect("Texture atlas should be added to its assets resource.");
    assert_eq!(
        asset_server.get_load_state(atlas.texture.clone()),
        LoadState::Loaded
    );
    assert_eq!(
        asset_server.get_load_state(texture_assets.player.clone()),
        LoadState::Loaded
    );
    assert_eq!(
        asset_server.get_load_state(texture_assets.tree.clone()),
        LoadState::Loaded
    );
    println!("Everything looks good!");
    println!("Quitting the application...");
    quit.send(AppExit);
}

fn print_progress(progress: Option<Res<ProgressCounter>>) {
    if let Some(progress) = progress {
        info!("Current progress: {:?}", progress.progress());
    }
}

#[derive(Component, Clone, Eq, PartialEq, Debug, Hash, Copy)]
enum MyStates {
    AssetLoading,
    Next,
}
