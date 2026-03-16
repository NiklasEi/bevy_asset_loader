use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Test that two independent state types can each have their own loading state,
/// running in parallel without interfering with each other.
#[test]
fn parallel_loading_states_for_different_state_types() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ))
    .init_state::<GameState>()
    .init_state::<UiState>()
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Ready)
            .load_collection::<GameAudio>(),
    )
    .add_loading_state(
        LoadingState::new(UiState::Loading)
            .continue_to_state(UiState::Ready)
            .load_collection::<UiAudio>(),
    )
    .insert_resource(CompletionTracker::default())
    .add_systems(
        Update,
        (
            track_game_ready.run_if(in_state(GameState::Ready)),
            track_ui_ready.run_if(in_state(UiState::Ready)),
            quit_when_both_done,
            timeout,
        ),
    )
    .run();
}

fn track_game_ready(_: Res<GameAudio>, mut tracker: ResMut<CompletionTracker>) {
    tracker.game_done = true;
}

fn track_ui_ready(_: Res<UiAudio>, mut tracker: ResMut<CompletionTracker>) {
    tracker.ui_done = true;
}

fn quit_when_both_done(tracker: Res<CompletionTracker>, mut exit: MessageWriter<AppExit>) {
    if tracker.game_done && tracker.ui_done {
        exit.write(AppExit::Success);
    }
}

#[derive(Resource, Default)]
struct CompletionTracker {
    game_done: bool,
    ui_done: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Ready,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum UiState {
    #[default]
    Loading,
    Ready,
}

#[derive(AssetCollection, Resource)]
struct GameAudio {
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct UiAudio {
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("Test did not complete within 30 seconds");
    }
}
