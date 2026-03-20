use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Test that `on_failure_continue_to_state` transitions to the failure state
/// when an asset fails to load, and then loading can be retried.
#[test]
fn failure_state_allows_retry() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
        .init_state::<MyState>()
        .add_loading_state(
            LoadingState::new(MyState::Loading)
                .continue_to_state(MyState::Ready)
                .on_failure_continue_to_state(MyState::Error)
                .load_collection::<MissingAssets>(),
        )
        .insert_resource(ErrorCount(0))
        .add_systems(OnEnter(MyState::Error), on_error)
        .add_systems(Update, timeout)
        .run();
}

fn on_error(
    mut count: ResMut<ErrorCount>,
    mut next_state: ResMut<NextState<MyState>>,
    mut exit: MessageWriter<AppExit>,
) {
    count.0 += 1;
    if count.0 >= 2 {
        // Confirmed failure state works on re-entry too
        exit.write(AppExit::Success);
    } else {
        // Retry loading (will fail again)
        next_state.set(MyState::Loading);
    }
}

#[derive(Resource)]
struct ErrorCount(u32);

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyState {
    #[default]
    Loading,
    Ready,
    Error,
}

#[derive(AssetCollection, Resource)]
struct MissingAssets {
    #[asset(path = "does/not/exist.ogg")]
    _missing: Handle<AudioSource>,
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("Test did not complete within 30 seconds");
    }
}
