use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_sub_state::<MainMenuState>()
        .add_loading_state(
            LoadingState::new(MainMenuState::Loading)
                .continue_to_state(MainMenuState::Game)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MainMenuState::Game), setup)
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
}

fn setup(mut commands: Commands, my_assets: Res<MyAssets>) {
    commands.spawn(AudioPlayer(my_assets.background.clone()));
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(my_assets.player.clone()));
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
    Game,
}
