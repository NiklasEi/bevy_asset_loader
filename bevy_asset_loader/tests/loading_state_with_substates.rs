use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Test loading states using substates with both parent and child having loading states.
#[test]
fn parent_and_child_loading_states() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ))
    .init_state::<ParentState>()
    .add_sub_state::<ChildState>()
    // Parent state loads GameAudio
    .add_loading_state(
        LoadingState::new(ParentState::Loading)
            .continue_to_state(ParentState::InGame)
            .load_collection::<GameAudio>(),
    )
    // Child substate loads MenuAudio when InGame
    .add_loading_state(
        LoadingState::new(ChildState::Loading)
            .continue_to_state(ChildState::Ready)
            .load_collection::<MenuAudio>(),
    )
    .insert_resource(TestTracker::default())
    .add_systems(
        Update,
        (
            check_child_ready.run_if(in_state(ChildState::Ready)),
            timeout,
        ),
    )
    .run();
}

fn check_child_ready(
    _game: Res<GameAudio>,
    _menu: Res<MenuAudio>,
    mut exit: MessageWriter<AppExit>,
) {
    // Both collections loaded successfully
    exit.write(AppExit::Success);
}

#[derive(Resource, Default)]
struct TestTracker {
    _parent_loaded: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum ParentState {
    #[default]
    Loading,
    InGame,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(ParentState = ParentState::InGame)]
enum ChildState {
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
struct MenuAudio {
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("Test did not complete within 30 seconds");
    }
}
