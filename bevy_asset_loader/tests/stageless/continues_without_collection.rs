use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use iyes_loopless::prelude::*;

#[test]
fn continues_without_collection() {
    App::new()
        .add_loopless_state(MyStates::Load)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_loading_state(LoadingState::new(MyStates::Load).continue_to_state(MyStates::Next))
        .init_resource::<TestState>()
        .add_enter_system(MyStates::Next, exit)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(MyStates::Load)
                .with_system(expect)
                .into(),
        )
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

struct TestState {
    wait_frames: usize,
}

impl Default for TestState {
    fn default() -> Self {
        TestState { wait_frames: 4 }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Next,
}
