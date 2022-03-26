#![allow(dead_code)]

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

#[cfg_attr(all(not(feature = "2d"), not(feature = "3d")), test)]
fn can_run_without_next_state() {
    let mut app = App::new();
    app.add_state(MyStates::Load)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default());

    AssetLoader::new(MyStates::Load)
        .with_collection::<MyAssets>()
        .build(&mut app);

    app.init_resource::<TestState>()
        .add_system_set(
            SystemSet::on_update(MyStates::Load)
                .with_system(timeout)
                .with_system(expect),
        )
        .run();
}

fn timeout(time: Res<Time>) {
    if time.seconds_since_startup() > 10. {
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
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
}
