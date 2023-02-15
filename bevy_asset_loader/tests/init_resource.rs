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
        not(feature = "progress_tracking")
    ),
    test
)]
fn init_resource() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default())
        .add_loading_state(
            LoadingState::new(MyStates::Load)
                .continue_to_state(MyStates::Next)
                .with_collection::<MyAssets>()
                .init_resource::<PostProcessed>(),
        )
        .add_state::<MyStates>()
        .add_system_set(SystemSet::on_update(MyStates::Load).with_system(timeout))
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(expect))
        .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

fn expect(collection: Option<Res<PostProcessed>>, mut exit: EventWriter<AppExit>) {
    if collection.is_none() {
        panic!("Post processed collection was not inserted");
    } else {
        exit.send(AppExit);
    }
}

#[allow(dead_code)]
#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

// this struct could e.g. contain TextureAtlas handles or anything else
// created from previously loaded assets
#[allow(dead_code)]
#[derive(Resource)]
struct PostProcessed {
    background: Handle<AudioSource>,
    // use other resources/add fields
    fuu: String,
}

impl FromWorld for PostProcessed {
    fn from_world(world: &mut World) -> Self {
        let assets = world
            .get_resource::<MyAssets>()
            .expect("MyAssets not loaded");
        PostProcessed {
            background: assets.background.clone(),
            fuu: "bar".to_owned(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Next,
}
