use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::*;

fn main() {
    App::new()
        .add_loopless_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .on_failure_continue_to_state(MyStates::ErrorScreen)
                .with_collection::<MyAssets>(),
        )
        .add_state(MyStates::AssetLoading)
        .add_system(timeout.run_in_state(MyStates::AssetLoading))
        .add_enter_system(MyStates::Next, fail)
        .add_enter_system(MyStates::ErrorScreen, ok)
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
    #[asset(path = "non-existing-file.ogg")]
    _non_existing_file: Handle<AudioSource>,
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
}

fn fail() {
    panic!("The library should have switched to the failure state!");
}

fn ok(mut quit: EventWriter<AppExit>) {
    println!("As expected, the library switched to the failure state");
    println!("Quitting the application...");
    quit.send(AppExit);
}

fn timeout(time: Res<Time>) {
    if time.seconds_since_startup() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
    ErrorScreen,
}
