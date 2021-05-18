use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoaderPlugin};
use bevy_kira_audio::{AudioPlugin, AudioSource};

#[test]
fn no_assets() {
    App::build()
        .add_state(MyStates::Load)
        .add_plugins(DefaultPlugins)
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(AudioPlugin)
        .add_system_set(SystemSet::on_update(MyStates::Load).with_system(timeout.system()))
        .add_system_set(
            SystemSet::on_enter(MyStates::Next).with_system(expect_asset_collection.system()),
        )
        .add_plugin(AssetLoaderPlugin::<MyAssets, _>::new(
            MyStates::Load,
            MyStates::Next,
        ))
        .run();
}

fn timeout(time: Res<Time>) {
    if time.seconds_since_startup() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

fn expect_asset_collection(collection: Option<Res<MyAssets>>, mut exit: EventWriter<AppExit>) {
    if collection.is_none() {
        panic!("Asset collection was not inserted");
    } else {
        exit.send(AppExit);
    }
}

#[allow(dead_code)]
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "walking.ogg")]
    walking: Handle<AudioSource>,
    #[asset(path = "flying.ogg")]
    flying: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Next,
}
