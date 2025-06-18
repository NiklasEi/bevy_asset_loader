use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn custom_asset_file_path() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../assets".to_owned(),
            ..default()
        },
        AudioPlugin::default(),
        StatesPlugin,
    ));
    app.init_state::<MyStates>();
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .load_collection::<PlopAudio>(),
    )
    .add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
    .add_systems(OnEnter(MyStates::Next), expect)
    .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 60. {
        panic!("The asset loader did not change the state in 60 seconds");
    }
}

fn expect(collection: Option<Res<PlopAudio>>, mut exit: EventWriter<AppExit>) {
    if collection.is_none() {
        panic!("The asset collection was not inserted");
    } else {
        exit.write(AppExit::Success);
    }
}

#[derive(AssetCollection, Resource)]
struct PlopAudio {
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Next,
}
