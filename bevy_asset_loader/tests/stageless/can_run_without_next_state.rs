use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use iyes_loopless::prelude::*;

#[test]
fn can_run_without_next_state() {
    App::new()
        .add_loopless_state(MyStates::Load)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default())
        .add_loading_state(LoadingState::new(MyStates::Load).with_collection::<MyAssets>())
        .init_resource::<TestState>()
        .add_system_set(
            ConditionSet::new()
                .run_in_state(MyStates::Load)
                .with_system(timeout)
                .with_system(expect)
                .into(),
        )
        .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 10. {
        panic!("The asset loader did not load the collection in 10 seconds");
    }
}

fn expect(
    collection: Option<Res<MyAssets>>,
    mut exit: EventWriter<AppExit>,
    mut test_state: ResMut<TestState>,
) {
    if collection.is_some() {
        if test_state.wait_frames_after_load == 0 {
            exit.send(AppExit);
            return;
        }
        test_state.wait_frames_after_load -= 1;
    }
}

#[derive(Resource)]
struct TestState {
    wait_frames_after_load: usize,
}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            wait_frames_after_load: 5,
        }
    }
}

#[allow(dead_code)]
#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
}
