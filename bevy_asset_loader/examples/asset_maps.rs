use bevy::asset::{AssetPath, LoadedUntypedAsset};
use bevy::prelude::*;
use bevy::utils::HashMap;
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
    file_name: HashMap<FileName, Handle<AudioSource>>,
    // `FileStem` as the key will use the file name without the extension
    #[asset(path = "audio", collection(mapped, typed))]
    file_stem: HashMap<FileStem, Handle<AudioSource>>,
    // `FileStem` as the key will use the file name without the extension
    // #[asset(paths("animated/Fox.glb#Animation0"), collection(mapped, typed))]
    // labels: HashMap<AssetLabel, Handle<AnimationClip>>,
    #[asset(path = "animated/Fox.glb#Animation0")]
    anim: Handle<AnimationClip>,

    // You can implement your own map key types
    #[asset(path = "audio", collection(mapped, typed))]
    custom: HashMap<MyAudio, Handle<AudioSource>>,
}

fn use_audio_assets(audio_assets: Res<AudioAssets>, asset_server: Res<AssetServer>) {
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
    // audio_assets.labels.get("Animation0").expect("");

    // let anim: Handle<LoadedUntypedAsset> = asset_server.load_untyped("animated/Fox.glb#Animation0");
    let path = audio_assets.anim.path().unwrap().label().unwrap();

    // custom key
    audio_assets
        .custom
        .get(&MyAudio::Plop)
        .expect("Can access audio asset with custom key");
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
