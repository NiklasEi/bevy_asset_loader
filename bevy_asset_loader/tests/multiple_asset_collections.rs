use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

#[test]
fn multiple_asset_collections() {
    let mut app = App::build();

    AssetLoader::new(MyStates::Load, MyStates::Next)
        .with_collection::<MyAssets>()
        .with_collection::<MyOtherAssets>()
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
    if time.seconds_since_startup() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

fn expect(
    collection: Option<Res<MyAssets>>,
    other_collection: Option<Res<MyAssets>>,
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
struct MyAssets {
    #[asset(path = "flying.ogg")]
    flying: Handle<AudioSource>,
}

#[allow(dead_code)]
#[derive(AssetCollection)]
struct MyOtherAssets {
    #[asset(path = "walking.ogg")]
    walking: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Load,
    Next,
}
