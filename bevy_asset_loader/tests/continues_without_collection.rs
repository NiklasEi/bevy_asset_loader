#![allow(dead_code, unused_imports)]

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

#[cfg(all(
    not(feature = "2d"),
    not(feature = "3d"),
    not(feature = "progress_tracking")
))]
#[test]
fn continues_without_collection() {
    App::new()
        .add_state::<MyStates>()
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_loading_state(LoadingState::new(MyStates::Load).continue_to_state(MyStates::Next))
        .init_resource::<TestState>()
        .add_system_set(SystemSet::on_update(MyStates::Load).with_system(expect))
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(exit))
        .run();
}

fn expect(mut test_state: ResMut<TestState>) {
    if test_state.wait_frames == 0 {
        panic!("The asset loader did not continue to the next state");
    }
    test_state.wait_frames -= 1;
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}

#[derive(Resource)]
struct TestState {
    wait_frames: usize,
}

impl Default for TestState {
    fn default() -> Self {
        TestState { wait_frames: 1 }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Next,
}
