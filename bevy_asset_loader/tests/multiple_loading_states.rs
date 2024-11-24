use bevy::app::AppExit;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn multiple_loading_states() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ));
    app.init_state::<MyStates>();
    app.add_loading_state(
        LoadingState::new(MyStates::Splash)
            .continue_to_state(MyStates::Load)
            .load_collection::<SplashAssets>(),
    )
    .add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Play)
            .load_collection::<MyAssets>()
            .load_collection::<MyOtherAssets>(),
    )
    .add_systems(Update, (quit.run_if(in_state(MyStates::Play)), timeout))
    .add_systems(OnEnter(MyStates::Load), use_splash_assets)
    .add_systems(OnEnter(MyStates::Play), use_loading_assets)
    .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("The app did not finish in 30 seconds");
    }
}

fn use_splash_assets(_splash_assets: Res<SplashAssets>) {
    // I could do something with the splash assets here
}

fn use_loading_assets(
    _my_assets: Res<MyAssets>,
    _my_other_assets: Res<MyOtherAssets>,
    _splash_assets: Res<SplashAssets>,
) {
    // I could do something with the `MyAssets` and `MyOtherAssets` collections here
}

fn quit(mut exit: EventWriter<AppExit>) {
    info!("Everything fine, quitting the app");
    exit.send(AppExit::Success);
}

#[derive(AssetCollection, Resource)]
#[allow(dead_code)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
#[allow(dead_code)]
struct MyOtherAssets {
    #[asset(path = "audio/yippee.ogg")]
    yippee: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
#[allow(dead_code)]
struct SplashAssets {
    #[asset(path = "audio/plop.ogg")]
    plop: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Splash,
    Load,
    Play,
}
