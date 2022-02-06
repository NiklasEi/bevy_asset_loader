#![allow(dead_code)]

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

#[cfg_attr(not(feature = "render"), test)]
fn multiple_asset_collections() {
    let mut app = App::new();

    AssetLoader::new(MyStates::Load)
        .continue_to_state(MyStates::Next)
        .with_collection::<PlopAudio>()
        .with_collection::<BackgroundAudio>()
        .build(&mut app);

    app.add_state(MyStates::Load)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default())
        .add_system_set(SystemSet::on_update(MyStates::Load).with_system(timeout.system()))
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(expect.system()))
        .run();
}

fn timeout(time: Res<Time>) {
    if time.seconds_since_startup() > 60. {
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

#[allow(dead_code)]
#[derive(AssetCollection)]
struct PlopAudio {
    #[asset(path = "audio/plop.ogg")]
    plop: Handle<AudioSource>,
}

#[allow(dead_code)]
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
