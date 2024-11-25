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

    app.add_systems(OnEnter(MainMenuState::Active), setup_main_menu);

    app.run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

fn setup_main_menu(mut commands: Commands, my_assets: Res<MyAssets>) {
    commands.spawn(AudioSourceBundle {
        source: my_assets.background.clone(),
        ..Default::default()
    });
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
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
