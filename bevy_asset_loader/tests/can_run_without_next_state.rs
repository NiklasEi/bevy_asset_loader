#![allow(dead_code, unused_imports)]

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

#[cfg(all(
    not(feature = "2d"),
    not(feature = "3d"),
    not(feature = "progress_tracking")
))]
#[test]
fn can_run_without_next_state() {
    App::new()
        .add_state::<MyStates>()
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default())
        .add_loading_state(LoadingState::new(MyStates::Load))
        .add_collection_to_loading_state::<_, MyAssets>(MyStates::Load)
        .init_resource::<TestState>()
        .add_systems(
            Update,
            (
                expect.run_if(in_state(MyStates::Load)),
                timeout.run_if(in_state(MyStates::Load)),
            ),
        )
        .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 30. {
        panic!("The asset loader did not load the collection in 30 seconds");
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
}
