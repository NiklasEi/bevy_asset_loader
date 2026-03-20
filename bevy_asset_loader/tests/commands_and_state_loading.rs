use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Test that commands-based loading and state-based loading can coexist.
/// The state-based loading loads `BackgroundAudio`, while a system uses
/// commands to load `PlopAudio` independently.
#[test]
fn commands_loading_alongside_state_loading() {
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
            .continue_to_state(MyState::Ready)
            .load_collection::<BackgroundAudio>(),
    )
    .insert_resource(CommandsLoadDone(false))
    .add_systems(Startup, start_commands_loading)
    .add_systems(
        Update,
        (check_both_done.run_if(in_state(MyState::Ready)), timeout),
    )
    .run();
}

fn start_commands_loading(mut commands: Commands) {
    commands.load_collection::<PlopAudio>().observe(
        |_: On<AssetCollectionLoaded<PlopAudio>>, mut done: ResMut<CommandsLoadDone>| {
            done.0 = true;
        },
    );
}

fn check_both_done(
    _bg: Res<BackgroundAudio>,
    commands_done: Res<CommandsLoadDone>,
    mut exit: MessageWriter<AppExit>,
) {
    // State-based loading is done (we're in Ready state and BackgroundAudio exists).
    // Check if commands-based loading is also done.
    if commands_done.0 {
        exit.write(AppExit::Success);
    }
}

#[derive(Resource)]
struct CommandsLoadDone(bool);

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyState {
    #[default]
    Loading,
    Ready,
}

#[derive(AssetCollection, Resource)]
struct BackgroundAudio {
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct PlopAudio {
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("Test did not complete within 30 seconds");
    }
}
