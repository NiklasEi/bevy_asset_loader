use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Test that re-entering a loading state works correctly.
/// The state transitions: Loading -> Game -> Loading -> Game -> Done
#[test]
fn loading_state_reentry() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ))
    .init_state::<MyState>()
    .add_loading_state(
        LoadingState::new(MyState::Loading)
            .continue_to_state(MyState::Game)
            .load_collection::<AudioAssets>(),
    )
    .insert_resource(ReentryCount(0))
    .add_systems(OnEnter(MyState::Game), on_enter_game)
    .add_systems(Update, timeout)
    .run();
}

fn on_enter_game(
    mut count: ResMut<ReentryCount>,
    _assets: Res<AudioAssets>,
    mut next_state: ResMut<NextState<MyState>>,
    mut exit: MessageWriter<AppExit>,
) {
    count.0 += 1;
    if count.0 < 3 {
        // Go back to Loading to test re-entry
        next_state.set(MyState::Loading);
    } else {
        // Done after 3 successful load cycles
        exit.write(AppExit::Success);
    }
}

#[derive(Resource)]
struct ReentryCount(u32);

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyState {
    #[default]
    Loading,
    Game,
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("Test did not complete within 30 seconds");
    }
}
