use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn continues_to_failure_state() {
    let mut app = App::new();
    app.init_state::<MyStates>();

    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
    #[cfg(feature = "progress_tracking")]
    app.add_plugins(iyes_progress::ProgressPlugin::new(MyStates::Load));
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .on_failure_continue_to_state(MyStates::Error)
            .load_collection::<Audio>(),
    )
    .add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
    .add_systems(OnEnter(MyStates::Next), fail)
    .add_systems(OnEnter(MyStates::Error), exit)
    .run();
}

fn fail() {
    panic!("The library should have switched to the failure state");
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit::Success);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(AssetCollection, Resource)]
struct Audio {
    #[asset(path = "audio/plop.ogg")]
    _no_loader_for_ogg_files: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Error,
    Next,
}
