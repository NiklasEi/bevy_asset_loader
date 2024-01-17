#![allow(dead_code, unused_imports)]

use bevy::app::AppExit;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[cfg(all(
    not(feature = "2d"),
    not(feature = "3d"),
    not(feature = "progress_tracking")
))]
#[test]
fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            AudioPlugin::default(),
        ))
        .add_state::<Prepare>()
        .add_state::<Game>()
        .add_loading_state(
            LoadingState::new(Game::Booting)
                .continue_to_state(Game::Loading)
                .load_collection::<GameStateCollection>(),
        )
        .add_loading_state(
            LoadingState::new(Prepare::Loading)
                .continue_to_state(Prepare::Finalize)
                .load_collection::<LoadingStateCollection>(),
        )
        .add_systems(Update, (quit.run_if(in_state(Game::Play)), timeout))
        .add_systems(
            OnEnter(Game::Loading),
            (go_to_loading_loading, probe_game_state),
        )
        .add_systems(OnEnter(Prepare::Finalize), go_to_game_play_loading_done)
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
enum Prepare {
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

fn go_to_loading_loading(mut state: ResMut<NextState<Prepare>>) {
    state.set(Prepare::Loading);
}

fn go_to_game_play_loading_done(
    mut game_state: ResMut<NextState<Game>>,
    mut loading_state: ResMut<NextState<Prepare>>,
) {
    game_state.set(Game::Play);
    loading_state.set(Prepare::Done);
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
