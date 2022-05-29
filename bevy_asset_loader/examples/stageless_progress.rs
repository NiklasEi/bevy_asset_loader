use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_loopless::prelude::*;
use iyes_progress::{ProgressCounter, ProgressPlugin};

/// This example shows how to track the loading progress of your collections using `iyes_progress`
///
/// Running it will print the current progress for every frame. The five assets from
/// the two collections will be loaded rather quickly (one or two frames). The final task
/// completes after one second. At that point, `iyes_progress` will continue to the next state
/// and the app will terminate.
fn main() {
    let mut app = App::new();
    app.add_loopless_state(MyStates::AssetLoading);
    AssetLoader::new(MyStates::AssetLoading)
        .with_collection::<TextureAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    app.add_plugins(DefaultPlugins)
        // track progress during `MyStates::AssetLoading` and continue to `MyStates::Next` when progress is completed
        .add_plugin(ProgressPlugin::new(MyStates::AssetLoading).continue_to(MyStates::Next))
        // gracefully quit the app when `MyStates::Next` is reached
        .add_enter_system(MyStates::Next, quit)
        .add_system(track_fake_long_task.run_in_state(MyStates::AssetLoading))
        .add_system_to_stage(CoreStage::PostUpdate, print_progress)
        .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct TextureAssets {
    #[asset(path = "images/player.png")]
    _player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    _tree: Handle<Image>,
    #[asset(path = "images/female_adventurer.png")]
    _female_adventurer: Handle<Image>,
}

fn track_fake_long_task(time: Res<Time>, progress: Res<ProgressCounter>) {
    if time.seconds_since_startup() > 1. {
        info!("done");
        progress.manually_track(true.into());
    } else {
        progress.manually_track(false.into());
    }
}

fn quit(mut quit: EventWriter<AppExit>) {
    info!("quitting");
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
