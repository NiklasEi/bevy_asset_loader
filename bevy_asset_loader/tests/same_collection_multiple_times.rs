use bevy::app::AppExit;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[test]
fn same_collection_multiple_times() {
    let mut app = App::new();
    app.init_state::<MyStates>();

    #[cfg(feature = "progress_tracking")]
    app.add_plugins(iyes_progress::ProgressPlugin::new(MyStates::Load));
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
    ))
    .add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Play)
            .load_collection::<MyAssets>()
            .load_collection::<MyAssets>(),
    )
    .configure_loading_state(LoadingStateConfig::new(MyStates::Load).load_collection::<MyAssets>())
    .add_systems(Update, (quit.run_if(in_state(MyStates::Play)), timeout))
    .add_systems(OnEnter(MyStates::Play), use_loading_assets)
    .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(Clone, Copy, Debug, States, Default, PartialEq, Eq, Hash)]
enum MyStates {
    #[default]
    Load,
    Play,
}

fn use_loading_assets(assets: Res<MyAssets>, asset_server: Res<AssetServer>) {
    assert!(
        asset_server.is_loaded_with_dependencies(&assets.background),
        "The collection is not fully loaded!"
    );
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 30. {
        panic!("The app did not finish in 30 seconds");
    }
}

fn quit(mut exit: EventWriter<AppExit>) {
    info!("Everything fine, quitting the app");
    exit.send(AppExit::Success);
}
