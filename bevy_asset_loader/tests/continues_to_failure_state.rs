use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn continues_to_failure_state() {
    run_until_failure_state::<Audio>(App::new());
}

/// The asset type of a typed folder is not registered, so loading must fail instead of panicking.
#[test]
fn continues_to_failure_state_for_unregistered_folder_type() {
    run_until_failure_state::<AudioFolder>(App::new());
}

/// The asset type of a dynamic asset is not registered, so loading must fail instead of panicking.
#[cfg(feature = "standard_dynamic_assets")]
#[test]
fn continues_to_failure_state_for_unregistered_dynamic_type() {
    let mut app = App::new();
    app.insert_resource({
        let mut dynamic_assets = DynamicAssets::default();
        dynamic_assets.register_asset(
            "plop",
            Box::new(StandardDynamicAsset::File {
                path: "audio/plop.ogg".to_owned(),
            }),
        );
        dynamic_assets
    });
    run_until_failure_state::<DynamicAudio>(app);
}

fn run_until_failure_state<C: AssetCollection>(mut app: App) {
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
    app.init_state::<MyStates>();
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .on_failure_continue_to_state(MyStates::Error)
            .load_collection::<C>(),
    )
    .add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
    .add_systems(OnEnter(MyStates::Next), fail)
    .add_systems(OnEnter(MyStates::Error), exit)
    .run();
}

fn fail() {
    panic!("The library should have switched to the failure state");
}

fn exit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(AssetCollection, Resource)]
struct Audio {
    #[asset(path = "audio/plop.ogg")]
    _no_loader_for_ogg_files: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct AudioFolder {
    #[asset(path = "audio", collection(typed))]
    _unregistered_type: Vec<Handle<AudioSource>>,
}

#[allow(dead_code)]
#[derive(AssetCollection, Resource)]
struct DynamicAudio {
    #[asset(key = "plop")]
    _unregistered_type: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Error,
    Next,
}
