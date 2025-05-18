use bevy::app::AppExit;
use bevy::asset::AssetPath;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<AudioAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), use_audio_assets)
        .run();
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    // A String as the key will use the full asset path
    #[asset(path = "audio", collection(mapped, typed))]
    full_path: HashMap<String, Handle<AudioSource>>,
    // `FileName` as the key will use the file name (strip away any directories in the path)
    #[asset(path = "audio", collection(mapped, typed))]
    file_name: HashMap<AssetFileName, Handle<AudioSource>>,
    // `FileStem` as the key will use the file name without the extension
    #[asset(path = "audio", collection(mapped, typed))]
    file_stem: HashMap<AssetFileStem, Handle<AudioSource>>,
    // `AssetLabel` as the key uses the label (part after the `#` here)
    // This will panic if an asset in the collection has no label!
    #[asset(
        paths(
            "animated/Fox.glb#Animation0",
            "animated/Fox.glb#Animation1",
            "animated/Fox.glb#Animation2"
        ),
        collection(mapped, typed)
    )]
    fox_animations: HashMap<AssetLabel, Handle<AnimationClip>>,

    // You can implement your own map key types
    #[asset(path = "audio", collection(mapped, typed))]
    custom: HashMap<MyAudio, Handle<AudioSource>>,
}

fn use_audio_assets(audio_assets: Res<AudioAssets>, mut quit: EventWriter<AppExit>) {
    audio_assets
        .full_path
        .get("audio/plop.ogg")
        .expect("Can access audio asset with full path");
    audio_assets
        .file_name
        .get("plop.ogg")
        .expect("Can access audio asset with file name");
    audio_assets
        .file_stem
        .get("plop")
        .expect("Can access audio asset with file stem");
    audio_assets
        .fox_animations
        .get("Animation0")
        .expect("Can access animation via its label");

    // custom key
    audio_assets
        .custom
        .get(&MyAudio::Plop)
        .expect("Can access audio asset with custom key");

    info!("Everything looks good!");
    info!("Quitting the application...");
    quit.write(AppExit::Success);
}

#[derive(PartialEq, Eq, Hash)]
enum MyAudio {
    Plop,
    Yippee,
    Background,
    Unknown(Box<str>),
}

impl MapKey for MyAudio {
    fn from_asset_path(path: &AssetPath) -> Self {
        let stem = path
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .expect("Path should be valid UTF-8")
            .to_string();
        match stem.as_str() {
            "plop" => MyAudio::Plop,
            "background" => MyAudio::Background,
            "yippee" => MyAudio::Yippee,
            n => MyAudio::Unknown(n.into()),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
