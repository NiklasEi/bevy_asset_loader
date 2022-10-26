#![allow(dead_code)]

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

#[cfg_attr(
    all(
        not(feature = "2d"),
        not(feature = "3d"),
        not(feature = "progress_tracking"),
        not(feature = "stageless")
    ),
    test
)]
fn multiple_asset_collections() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default())
        .add_loading_state(
            LoadingState::new(MyStates::Load)
                .continue_to_state(MyStates::Next)
                .with_collection::<PlopAudio>()
                .with_collection::<BackgroundAudio>(),
        )
        .add_state(MyStates::Load)
        .add_system_set(SystemSet::on_update(MyStates::Load).with_system(timeout))
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(expect))
        .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 60. {
        panic!("The asset loader did not change the state in 60 seconds");
    }
}

fn expect(
    collection: Option<Res<PlopAudio>>,
    other_collection: Option<Res<BackgroundAudio>>,
    mut exit: EventWriter<AppExit>,
) {
    if collection.is_none() || other_collection.is_none() {
        panic!("At least one asset collection was not inserted");
    } else {
        exit.send(AppExit);
    }
}

#[derive(AssetCollection)]
struct PlopAudio {
    #[asset(path = "audio/plop.ogg")]
    plop: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct BackgroundAudio {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Next,
}
