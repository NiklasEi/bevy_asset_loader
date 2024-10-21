use bevy::app::AppExit;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.init_state::<AppState>();
    app.add_sub_state::<MainMenuState>();

    app.add_loading_state(
        LoadingState::new(MainMenuState::Loading)
            .continue_to_state(MainMenuState::Active)
            .on_failure_continue_to_state(MainMenuState::Error)
            .load_collection::<MyAssets>(),
    );

    app.add_systems(OnEnter(AppState::Booting), booting)
        .add_systems(
            Update,
            finish_booting
                .run_if(in_state(AppState::Booting).and_then(input_just_pressed(KeyCode::Space))),
        );

    app.add_systems(Update, timeout.run_if(in_state(MainMenuState::Loading)))
        .add_systems(OnEnter(MainMenuState::Active), fail)
        .add_systems(OnEnter(MainMenuState::Error), ok);

    app.run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
    #[asset(path = "non-existing-file.ogg")]
    _non_existing_file: Handle<AudioSource>,
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
}

fn fail() {
    panic!("The library should have switched to the failure state!");
}

fn ok(mut quit: EventWriter<AppExit>) {
    info!("As expected, bevy_asset_loader switched to the failure state");
    info!("Quitting the application...");
    quit.send(AppExit::Success);
}

fn booting() {
    info!("Booting...press space to finish booting");
}

fn finish_booting(mut state: ResMut<NextState<AppState>>) {
    state.set(AppState::MainMenu);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_seconds_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    Booting,
    MainMenu,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::MainMenu)]
pub enum MainMenuState {
    #[default]
    Loading,
    Error,
    Active,
}
