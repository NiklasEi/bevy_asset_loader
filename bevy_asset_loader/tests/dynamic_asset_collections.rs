#![cfg(all(feature = "2d", feature = "standard_dynamic_assets"))]

use bevy::app::AppExit;
use bevy::audio::AudioLoader;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;
use std::path::PathBuf;

const MIXED: &str = r#"({
    "single_layout": TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 4, rows: 1),
    "jumps": [
        TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 16, rows: 1),
        TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 12, rows: 1),
        TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 8, rows: 1),
    ],
    "named": {
        "walk": TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 3, rows: 1),
        "idle": TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 5, rows: 1),
    },
})"#;

// A file written for the removed `StandardDynamicAssetArrayCollection`
const LEGACY_ARRAYS: &str = r#"({
    "runs": [
        TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 6, rows: 1),
        TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 2, rows: 1),
    ],
})"#;

const INVALID: &str = r#"({
    "jumps": [
        TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, columns: 16, rows: 1),
        TextureAtlasLayout(tile_size_x: 24, tile_size_y: 32, colums: 12, rows: 1),
    ],
})"#;

const SOUND_MAP: &str = r#"({
    "sounds": {
        "folder": Folder(path: "audio"),
        "files": Files(paths: ["audio/plop.ogg", "audio/yippee.ogg"]),
        "file": File(path: "audio/background.ogg"),
    },
})"#;

#[test]
fn single_dynamic_assets_arrays_and_maps() {
    let assets = asset_dir("single_dynamic_assets_arrays_and_maps");
    let mut app = app(&assets);
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .on_failure_continue_to_state(MyStates::Failed)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("mixed.assets.ron")
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("legacy.asset_arrays.ron")
            .load_collection::<Layouts>(),
    )
    .add_systems(OnEnter(MyStates::Next), check_layouts)
    .add_systems(OnEnter(MyStates::Failed), || -> () {
        panic!("Loading the dynamic asset files failed")
    });
    assert_eq!(app.run(), AppExit::Success);
}

#[test]
fn invalid_entry_in_array_fails_loading() {
    expect_failure("invalid_entry_in_array_fails_loading", "invalid.assets.ron");
}

#[test]
fn folders_and_files_in_map() {
    let assets = asset_dir("folders_and_files_in_map");
    let mut app = app(&assets);
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .on_failure_continue_to_state(MyStates::Failed)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("sounds.assets.ron")
            .load_collection::<Sounds>(),
    )
    .add_systems(OnEnter(MyStates::Next), check_sounds)
    .add_systems(OnEnter(MyStates::Failed), || -> () {
        panic!("Loading the sounds failed")
    });
    assert_eq!(app.run(), AppExit::Success);
}

#[test]
#[should_panic(expected = "but the field expects a single handle per key")]
fn folder_in_map_of_single_handles_panics() {
    let assets = asset_dir("folder_in_map_of_single_handles_panics");
    let mut app = app(&assets);
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("sounds.assets.ron")
            .load_collection::<SingleSounds>(),
    );
    app.run();
}

#[test]
#[should_panic(
    expected = "Cannot load the field `named` of the asset collection ListOfNamedLayouts: the field expects a list of handles, but the dynamic asset 'named' is a map with 2 entries (idle, walk)."
)]
fn list_field_on_map_panics() {
    let assets = asset_dir("list_field_on_map_panics");
    let mut app = app(&assets);
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("mixed.assets.ron")
            .load_collection::<ListOfNamedLayouts>(),
    );
    app.run();
}

#[test]
#[should_panic(
    expected = "Cannot load the field `_run` of the asset collection MissingEntry: the dynamic asset 'named' has no entry 'run'."
)]
fn missing_entry_panics() {
    let assets = asset_dir("missing_entry_panics");
    let mut app = app(&assets);
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("mixed.assets.ron")
            .load_collection::<MissingEntry>(),
    );
    app.run();
}

#[derive(AssetCollection, Resource)]
struct MissingEntry {
    #[asset(key = "named[run]")]
    _run: Handle<TextureAtlasLayout>,
}

fn expect_failure(test: &str, file: &str) {
    let assets = asset_dir(test);
    let mut app = app(&assets);
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .on_failure_continue_to_state(MyStates::Failed)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(file)
            .load_collection::<Layouts>(),
    )
    .add_systems(OnEnter(MyStates::Next), || -> () {
        panic!("Loading an invalid dynamic asset file succeeded")
    })
    .add_systems(
        OnEnter(MyStates::Failed),
        |mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        },
    );
    assert_eq!(app.run(), AppExit::Success);
}

fn asset_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bevy_asset_loader_{test}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mixed.assets.ron"), MIXED).unwrap();
    std::fs::write(dir.join("legacy.asset_arrays.ron"), LEGACY_ARRAYS).unwrap();
    std::fs::write(dir.join("invalid.assets.ron"), INVALID).unwrap();
    std::fs::write(dir.join("sounds.assets.ron"), SOUND_MAP).unwrap();
    let audio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/audio");
    std::fs::create_dir_all(dir.join("audio")).unwrap();
    for file in std::fs::read_dir(audio).unwrap() {
        let file = file.unwrap();
        std::fs::copy(file.path(), dir.join("audio").join(file.file_name())).unwrap();
    }
    dir
}

fn app(assets: &std::path::Path) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: assets.to_string_lossy().into_owned(),
            ..default()
        },
        StatesPlugin,
    ))
    .init_asset::<AudioSource>()
    .init_asset_loader::<AudioLoader>()
    .init_asset::<TextureAtlasLayout>()
    .init_state::<MyStates>()
    .add_systems(Update, timeout);
    app
}

#[derive(AssetCollection, Resource)]
struct Layouts {
    #[asset(key = "single_layout")]
    single: Handle<TextureAtlasLayout>,
    #[asset(key = "jumps", collection(typed))]
    jumps: Vec<Handle<TextureAtlasLayout>>,
    #[asset(key = "runs", collection(typed))]
    runs: Vec<Handle<TextureAtlasLayout>>,
    #[asset(key = "named", collection(mapped, typed))]
    named: HashMap<String, Handle<TextureAtlasLayout>>,
    #[asset(key = "named", collection(mapped))]
    named_untyped: HashMap<AssetFileStem, UntypedHandle>,
    #[asset(key = "named[walk]")]
    walk: Handle<TextureAtlasLayout>,
    #[asset(key = "jumps[1]")]
    second_jump: Handle<TextureAtlasLayout>,
    #[asset(key = "named[run]", optional)]
    run: Option<Handle<TextureAtlasLayout>>,
}

fn check_layouts(
    collection: Res<Layouts>,
    layouts: Res<Assets<TextureAtlasLayout>>,
    mut exit: MessageWriter<AppExit>,
) {
    let tiles = |handle: &Handle<TextureAtlasLayout>| layouts.get(handle).unwrap().textures.len();
    assert_eq!(tiles(&collection.single), 4);
    assert_eq!(
        collection.jumps.iter().map(tiles).collect::<Vec<_>>(),
        vec![16, 12, 8]
    );
    assert_eq!(
        collection.runs.iter().map(tiles).collect::<Vec<_>>(),
        vec![6, 2]
    );
    assert_eq!(collection.named.len(), 2);
    assert_eq!(tiles(&collection.named["walk"]), 3);
    assert_eq!(tiles(&collection.named["idle"]), 5);
    assert_eq!(collection.named_untyped.len(), 2);
    assert_eq!(tiles(&collection.named_untyped["idle"].clone().typed()), 5);
    assert_eq!(tiles(&collection.walk), 3);
    assert_eq!(tiles(&collection.second_jump), 12);
    assert_eq!(collection.run, None);
    exit.write(AppExit::Success);
}

#[derive(AssetCollection, Resource)]
struct Sounds {
    #[asset(key = "sounds[folder]", collection(typed))]
    folder: Vec<Handle<AudioSource>>,
    #[asset(key = "sounds[file]")]
    file: Handle<AudioSource>,
    #[asset(key = "sounds", collection(mapped, typed))]
    typed: HashMap<String, Vec<Handle<AudioSource>>>,
    #[asset(key = "sounds", collection(mapped))]
    untyped: HashMap<AssetFileStem, Vec<UntypedHandle>>,
}

#[derive(AssetCollection, Resource)]
struct ListOfNamedLayouts {
    #[asset(key = "named", collection(typed))]
    named: Vec<Handle<TextureAtlasLayout>>,
}

#[derive(AssetCollection, Resource)]
struct SingleSounds {
    #[asset(key = "sounds", collection(mapped, typed))]
    _sounds: HashMap<String, Handle<AudioSource>>,
}

fn check_sounds(
    sounds: Res<Sounds>,
    asset_server: Res<AssetServer>,
    audio: Res<Assets<AudioSource>>,
    mut exit: MessageWriter<AppExit>,
) {
    let paths = |handles: &[Handle<AudioSource>]| {
        let mut paths: Vec<_> = handles
            .iter()
            .map(|handle| {
                assert!(audio.contains(handle));
                asset_server.get_path(handle).unwrap().to_string()
            })
            .collect();
        paths.sort();
        paths
    };
    assert_eq!(
        paths(&sounds.typed["folder"]),
        vec!["audio/background.ogg", "audio/plop.ogg", "audio/yippee.ogg"]
    );
    assert_eq!(
        paths(&sounds.typed["files"]),
        vec!["audio/plop.ogg", "audio/yippee.ogg"]
    );
    assert_eq!(paths(&sounds.typed["file"]), vec!["audio/background.ogg"]);
    assert_eq!(
        paths(&sounds.folder),
        vec!["audio/background.ogg", "audio/plop.ogg", "audio/yippee.ogg"]
    );
    assert_eq!(
        paths(std::slice::from_ref(&sounds.file)),
        vec!["audio/background.ogg"]
    );
    assert_eq!(sounds.untyped.len(), 3);
    assert_eq!(sounds.untyped["folder"].len(), 3);
    assert_eq!(sounds.untyped["files"].len(), 2);
    assert_eq!(sounds.untyped["file"].len(), 1);
    exit.write(AppExit::Success);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 10. {
        panic!("The app did not finish in 10 seconds");
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Next,
    Failed,
}
