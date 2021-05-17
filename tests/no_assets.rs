use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_assets_loader::{AssetCollection, AssetLoaderPlugin};
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

struct MyAssets {
    walking: Handle<AudioSource>,
}

impl AssetCollection for MyAssets {
    fn create(asset_server: &mut ResMut<AssetServer>) -> Self {
        MyAssets {
            walking: asset_server.get_handle("walking.ogg"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Next,
}
