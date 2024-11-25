use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn can_run_with_sub_states() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            AudioPlugin::default(),
            StatesPlugin,
        ))
        .init_state::<AppState>()
        .add_sub_state::<MainMenuState>()
        .add_loading_state(
            LoadingState::new(MainMenuState::Loading)
                .continue_to_state(MainMenuState::Active)
                .load_collection::<MyAssets>(),
        )
        .init_resource::<TestState>()
        .add_systems(
            Update,
            (
                load_main_menu.run_if(in_state(AppState::Load)),
                expect.run_if(in_state(MainMenuState::Active)),
                timeout.run_if(in_state(MainMenuState::Loading)),
            ),
        )
        .run();
}

fn load_main_menu(mut state: ResMut<NextState<AppState>>) {
    state.set(AppState::MainMenu);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("The asset loader did not load the collection in 30 seconds");
    }
}

fn expect(
    collection: Option<Res<MyAssets>>,
    mut exit: EventWriter<AppExit>,
    mut test_state: ResMut<TestState>,
) {
    if collection.is_some() {
        if test_state.wait_frames_after_load == 0 {
            exit.send(AppExit::Success);
            return;
        }
        test_state.wait_frames_after_load -= 1;
    }
}

#[derive(Resource)]
struct TestState {
    wait_frames_after_load: usize,
}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            wait_frames_after_load: 5,
        }
    }
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AppState {
    #[default]
    Load,
    MainMenu,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::MainMenu)]
enum MainMenuState {
    #[default]
    Loading,
    Active,
}
