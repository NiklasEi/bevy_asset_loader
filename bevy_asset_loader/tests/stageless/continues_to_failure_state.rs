use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use iyes_loopless::prelude::*;

#[test]
fn continues_to_failure_state() {
    App::new()
        .add_loopless_state(MyStates::Load)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_loading_state(
            LoadingState::new(MyStates::Load)
                .continue_to_state(MyStates::Next)
                .on_failure_continue_to_state(MyStates::Error)
                .with_collection::<Audio>(),
        )
        .add_system(timeout.run_in_state(MyStates::Load))
        .add_enter_system(MyStates::Next, fail)
        .add_enter_system(MyStates::Error, exit)
        .run();
}

fn fail() {
    panic!("The library should have switched to the failure state");
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(AssetCollection, Resource)]
struct Audio {
    #[asset(path = "audio/plop.ogg")]
    _no_loader_for_ogg_files: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Error,
    Next,
}
