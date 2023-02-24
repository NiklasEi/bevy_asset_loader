use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_state::<MyStates>()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .on_failure_continue_to_state(MyStates::ErrorScreen),
        )
        .add_collection_to_loading_state::<_, MyAssets>(MyStates::AssetLoading)
        .add_system(timeout.run_if(in_state(MyStates::AssetLoading)))
        .add_system(fail.in_schedule(OnEnter(MyStates::Next)))
        .add_system(ok.in_schedule(OnEnter(MyStates::ErrorScreen)))
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
    info!("As expected, bevy_asset_loader switched to the failure state");
    info!("Quitting the application...");
    quit.send(AppExit);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
    ErrorScreen,
}
