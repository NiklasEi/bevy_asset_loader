use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn mapped_path_use_slash() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ));
    app.init_state::<MyStates>();
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .load_collection::<AudioCollection>(),
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

fn expect(collection: Option<Res<AudioCollection>>, mut exit: EventWriter<AppExit>) {
    if collection.is_none() {
        panic!("At least one asset collection was not inserted");
    } else if let Some(collection) = collection {
        // make sure the asset paths use slash on all OS
        assert_eq!(
            &collection.single_file.clone().path().unwrap().to_string(),
            "audio/yippee.ogg"
        );
        let files = &collection.files;
        assert!(
            files.contains_key("audio/plop.ogg"),
            "Expected path 'audio/plop.ogg' was not in {:?}",
            files
        );
        exit.write(AppExit::Success);
    }
}

#[derive(AssetCollection, Resource, Debug)]
struct AudioCollection {
    #[asset(path = "audio", collection(typed, mapped))]
    files: HashMap<String, Handle<AudioSource>>,
    #[asset(path = "audio/yippee.ogg")]
    single_file: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Next,
}
