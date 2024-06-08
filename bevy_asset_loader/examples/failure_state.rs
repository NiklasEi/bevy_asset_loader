use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .init_state::<MyStates>()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .on_failure_continue_to_state(MyStates::ErrorScreen)
                .load_collection::<MyAssets>(),
        )
        .add_systems(Update, timeout.run_if(in_state(MyStates::AssetLoading)))
        .add_systems(OnEnter(MyStates::Next), fail)
        .add_systems(OnEnter(MyStates::ErrorScreen), ok)
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
    quit.send(AppExit::Success);
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
