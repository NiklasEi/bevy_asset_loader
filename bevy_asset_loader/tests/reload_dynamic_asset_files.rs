#![allow(dead_code, unused_imports)]

use bevy::app::AppExit;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[cfg(feature = "standard_dynamic_assets")]
#[test]
fn main() {
    let mut app = App::new();
    app.init_state::<MyStates>();

    #[cfg(feature = "progress_tracking")]
    app.add_plugins((
        iyes_progress::ProgressPlugin::new(MyStates::SplashAssetLoading),
        iyes_progress::ProgressPlugin::new(MyStates::MainMenuAssetLoading),
    ));
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
    ))
    .insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)))
    .add_loading_state(
        LoadingState::new(MyStates::SplashAssetLoading)
            .continue_to_state(MyStates::Splash)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "full_dynamic_collection.assets.ron",
            ),
    )
    .add_systems(Update, splash_countdown.run_if(in_state(MyStates::Splash)))
    .add_loading_state(
        LoadingState::new(MyStates::MainMenuAssetLoading)
            .continue_to_state(MyStates::MainMenu)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "full_dynamic_collection.assets.ron",
            )
            .load_collection::<MainMenuAssets>(),
    )
    .add_systems(Update, (timeout, quit.run_if(in_state(MyStates::MainMenu))))
    .run();
}

#[derive(AssetCollection, Resource)]
struct SplashAssets {
    #[asset(key = "single_file")]
    _another_file: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct MainMenuAssets {
    #[asset(key = "single_file")]
    _single_file: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    SplashAssetLoading,
    Splash,
    MainMenuAssetLoading,
    MainMenu,
}

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

fn splash_countdown(
    mut game_state: ResMut<NextState<MyStates>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(MyStates::MainMenuAssetLoading);
    }
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 30. {
        panic!("The app did not finish in 30 seconds");
    }
}

fn quit(mut exit: EventWriter<AppExit>) {
    info!("Everything fine, quitting the app");
    exit.send(AppExit);
}
