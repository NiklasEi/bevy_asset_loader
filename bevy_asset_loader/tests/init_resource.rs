use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn init_resource() {
    let mut app = App::new();
    app.init_state::<MyStates>();

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ));
    #[cfg(feature = "progress_tracking")]
    app.add_plugins(iyes_progress::ProgressPlugin::new(MyStates::Load));
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .load_collection::<MyAssets>()
            .init_resource::<PostProcessed>(),
    )
    .add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
    .add_systems(OnEnter(MyStates::Next), expect)
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
        exit.send(AppExit::Success);
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
