use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .on_failure_continue_to_state(MyStates::ErrorScreen)
                .with_collection::<MyAssets>(),
        )
        .add_state(MyStates::AssetLoading)
        .add_system_set(SystemSet::on_update(MyStates::AssetLoading).with_system(timeout))
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(fail))
        .add_system_set(SystemSet::on_enter(MyStates::ErrorScreen).with_system(ok))
        .run();
}

#[derive(AssetCollection, Resource)]
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
    if time.elapsed_seconds_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
    ErrorScreen,
}
