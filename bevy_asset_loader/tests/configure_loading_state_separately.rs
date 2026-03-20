use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Test that `configure_loading_state` works when adding collections after `add_loading_state`.
#[test]
fn configure_loading_state_adds_collections() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ))
    .init_state::<MyState>()
    .add_loading_state(LoadingState::new(MyState::Loading).continue_to_state(MyState::Ready))
    // Add collections separately via configure_loading_state
    .configure_loading_state(
        LoadingStateConfig::new(MyState::Loading).load_collection::<BackgroundAudio>(),
    )
    .configure_loading_state(
        LoadingStateConfig::new(MyState::Loading).load_collection::<PlopAudio>(),
    )
    .add_systems(OnEnter(MyState::Ready), verify_and_exit)
    .add_systems(Update, timeout)
    .run();
}

fn verify_and_exit(
    _bg: Res<BackgroundAudio>,
    _plop: Res<PlopAudio>,
    mut exit: MessageWriter<AppExit>,
) {
    exit.write(AppExit::Success);
}

/// Test that `configure_loading_state` with `finally_init_resource` works.
#[test]
fn configure_loading_state_with_finally_init_resource() {
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
    .configure_loading_state(
        LoadingStateConfig::new(MyState::Loading).finally_init_resource::<DerivedResource>(),
    )
    .add_systems(OnEnter(MyState::Ready), verify_derived_and_exit)
    .add_systems(Update, timeout)
    .run();
}

fn verify_derived_and_exit(
    _bg: Res<BackgroundAudio>,
    derived: Res<DerivedResource>,
    mut exit: MessageWriter<AppExit>,
) {
    assert!(derived.initialized);
    exit.write(AppExit::Success);
}

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

#[derive(Resource)]
struct DerivedResource {
    initialized: bool,
}

impl FromWorld for DerivedResource {
    fn from_world(_world: &mut World) -> Self {
        DerivedResource { initialized: true }
    }
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("Test did not complete within 30 seconds");
    }
}
