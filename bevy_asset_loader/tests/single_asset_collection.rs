use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

#[test]
fn single_asset_collection() {
    let mut app = App::build();
    app.add_state(MyStates::Load)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default());
    AssetLoader::new(MyStates::Load, MyStates::Next)
        .with_collection::<MyAssets>()
        .build(&mut app);
    app.add_system_set(SystemSet::on_update(MyStates::Load).with_system(timeout.system()))
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(expect.system()))
        .run();
}

fn timeout(time: Res<Time>) {
    if time.seconds_since_startup() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

fn expect(collection: Option<Res<MyAssets>>, mut exit: EventWriter<AppExit>) {
    if collection.is_none() {
        panic!("Asset collection was not inserted");
    } else {
        exit.send(AppExit);
    }
}

#[allow(dead_code)]
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "flying.ogg")]
    flying: Handle<AudioSource>,
    #[asset(path = "walking.png")]
    walking: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Next,
}
