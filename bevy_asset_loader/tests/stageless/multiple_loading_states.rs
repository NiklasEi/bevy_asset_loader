use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use iyes_loopless::prelude::*;

#[test]
fn multiple_loading_states() {
    App::new()
        .add_loopless_state(MyStates::Splash)
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default())
        .add_loading_state(
            LoadingState::new(MyStates::Splash)
                .continue_to_state(MyStates::Load)
                .with_collection::<SplashAssets>(),
        )
        .add_loading_state(
            LoadingState::new(MyStates::Load)
                .continue_to_state(MyStates::Play)
                .with_collection::<MyOtherAssets>(),
        )
        .add_loading_state(
            LoadingState::new(MyStates::Load)
                .continue_to_state(MyStates::Play)
                .with_collection::<MyAssets>(),
        )
        .add_system(timeout)
        .add_enter_system(MyStates::Load, use_splash_assets)
        .add_enter_system(MyStates::Play, use_loading_assets)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(MyStates::Play)
                .with_system(quit)
                .into(),
        )
        .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 30. {
        panic!("The app did not finish in 30 seconds");
    }
}

fn use_splash_assets(_splash_assets: Res<SplashAssets>) {
    // I could do something with the splash assets here
}

fn use_loading_assets(_my_assets: Res<MyAssets>, _my_other_assets: Res<MyOtherAssets>) {
    // I could do something with the `MyAssets` and `MyOtherAssets` collections here
}

fn quit(mut exit: EventWriter<AppExit>) {
    println!("Everything fine, quitting the app");
    exit.send(AppExit);
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
    #[asset(path = "audio/yipee.ogg")]
    yipee: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
#[allow(dead_code)]
struct SplashAssets {
    #[asset(path = "audio/plop.ogg")]
    plop: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Splash,
    Load,
    Play,
}
