#![allow(dead_code, unused_imports)]

use bevy::app::AppExit;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};

#[cfg(all(
    not(feature = "2d"),
    not(feature = "3d"),
    not(feature = "progress_tracking")
))]
#[test]
fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(AudioPlugin::default())
        .add_state::<Loading>()
        .add_state::<Game>()
        .add_loading_state(LoadingState::new(Game::Booting).continue_to_state(Game::Loading))
        .add_collection_to_loading_state::<_, GameStateCollection>(Game::Booting)
        .add_loading_state(LoadingState::new(Loading::Loading).continue_to_state(Loading::Finalize))
        .add_collection_to_loading_state::<_, LoadingStateCollection>(Loading::Loading)
        .add_systems(Update, (quit.run_if(in_state(Game::Play)), timeout))
        .add_systems(
            OnEnter(Game::Loading),
            (go_to_loading_loading, probe_game_state),
        )
        .add_systems(OnEnter(Loading::Finalize), go_to_game_play_loading_done)
        .add_systems(OnEnter(Game::Play), probe_loading_state)
        .run();
}

#[derive(Clone, Copy, Debug, States, Default, PartialEq, Eq, Hash)]
enum Game {
    #[default]
    Booting,
    Loading,
    Play,
}

#[derive(Clone, Copy, Debug, States, Default, PartialEq, Eq, Hash)]
enum Loading {
    #[default]
    Done,
    Loading,
    Finalize,
}

#[derive(Resource, AssetCollection)]
pub struct GameStateCollection {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

fn probe_game_state(_res: Res<GameStateCollection>) {}

#[derive(Resource, AssetCollection)]
pub struct LoadingStateCollection {
    #[asset(path = "audio/yipee.ogg")]
    yipee: Handle<AudioSource>,
}

fn probe_loading_state(_res: Res<LoadingStateCollection>) {}

fn go_to_loading_loading(mut state: ResMut<NextState<Loading>>) {
    state.set(Loading::Loading);
}

fn go_to_game_play_loading_done(
    mut game_state: ResMut<NextState<Game>>,
    mut loading_state: ResMut<NextState<Loading>>,
) {
    game_state.set(Game::Play);
    loading_state.set(Loading::Done);
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
