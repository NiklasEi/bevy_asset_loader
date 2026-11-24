use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_asset_loader::prelude::*;

const WIDTH: f32 = 1024.;
const PLAYER_SPEED: f32 = 300.;
const BIOMES: [&str; 3] = [
    "images/biomes/forest.png",
    "images/biomes/grassland.png",
    "images/biomes/fall.png",
];

/// This example loads the assets of one biome at a time and unloads the previous ones.
///
/// Walk out of the screen to the right or left (arrow keys or A/D) to switch to the next or previous biome.
/// Every switch goes through the loading state again, which loads `BiomeAssets` for the new biome.
/// When loading is done, the new `BiomeAssets` replace the old ones. Together with the despawned
/// background entity, that drops the last handle to the old background and Bevy unloads the asset.
/// Watch the console for the loaded and unloaded images.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WIDTH as u32, 640),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<Biome>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                .load_collection::<BiomeAssets>(),
        )
        .init_collection::<PlayerAssets>()
        .add_systems(Startup, spawn_camera_and_player)
        .add_systems(OnEnter(GameState::Loading), choose_biome_assets)
        .add_systems(OnEnter(GameState::Playing), spawn_background)
        .add_systems(Update, walk.run_if(in_state(GameState::Playing)))
        .add_systems(Update, log_image_events)
        .run();
}

#[derive(AssetCollection, Resource)]
struct BiomeAssets {
    #[asset(key = "background")]
    background: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
struct PlayerAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
}

#[derive(Resource, Default)]
struct Biome(usize);

fn choose_biome_assets(biome: Res<Biome>, mut dynamic_assets: ResMut<DynamicAssets>) {
    dynamic_assets.register_asset(
        "background",
        Box::new(StandardDynamicAsset::File {
            path: BIOMES[biome.0].to_owned(),
        }),
    );
}

fn spawn_background(mut commands: Commands, biome_assets: Res<BiomeAssets>) {
    commands.spawn((
        Sprite::from_image(biome_assets.background.clone()),
        DespawnOnExit(GameState::Playing),
    ));
}

fn log_image_events(
    mut events: MessageReader<AssetEvent<Image>>,
    asset_server: Res<AssetServer>,
    mut paths: Local<HashMap<AssetId<Image>, String>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                if let Some(path) = asset_server.get_path(*id) {
                    info!("Loaded {path}");
                    paths.insert(*id, path.to_string());
                }
            }
            AssetEvent::Removed { id } => {
                if let Some(path) = paths.remove(id) {
                    info!("Unloaded {path}");
                }
            }
            _ => {}
        }
    }
}

// Rest of the example setup

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Component)]
struct Player;

fn spawn_camera_and_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_image(player_assets.player.clone()),
        Transform::from_xyz(0., -160., 1.),
        Player,
    ));
}

fn walk(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Single<&mut Transform, With<Player>>,
    mut biome: ResMut<Biome>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut direction = 0.;
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += 1.;
    }
    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= 1.;
    }
    player.translation.x += direction * PLAYER_SPEED * time.delta_secs();

    let edge = WIDTH / 2.;
    if player.translation.x > edge {
        player.translation.x = -edge;
        biome.0 = (biome.0 + 1) % BIOMES.len();
        next_state.set(GameState::Loading);
    } else if player.translation.x < -edge {
        player.translation.x = edge;
        biome.0 = (biome.0 + BIOMES.len() - 1) % BIOMES.len();
        next_state.set(GameState::Loading);
    }
}
