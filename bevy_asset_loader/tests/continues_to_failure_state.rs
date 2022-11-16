#![allow(dead_code, unused_imports)]

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

#[cfg(all(
    not(feature = "2d"),
    not(feature = "3d"),
    not(feature = "progress_tracking"),
    not(feature = "stageless")
))]
#[test]
fn continues_to_failure_state() {
    App::new()
        .add_state(MyStates::Load)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_loading_state(
            LoadingState::new(MyStates::Load)
                .continue_to_state(MyStates::Next)
                .on_failure_continue_to_state(MyStates::Error)
                .with_collection::<Audio>(),
        )
        .add_system_set(SystemSet::on_update(MyStates::Load).with_system(timeout))
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(fail))
        .add_system_set(SystemSet::on_enter(MyStates::Error).with_system(exit))
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
    no_loader_for_ogg_files: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Error,
    Next,
}
