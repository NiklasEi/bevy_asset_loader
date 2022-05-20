use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<MyAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading).run();
}

#[allow(dead_code)]
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "audio/background.ogg")]
    single_file: Handle<AudioSource>,
    #[asset(path = "images/player.png")]
    standard_material: Handle<StandardMaterial>,
    #[asset(path = "images/female_adventurer_sheet.png")]
    texture_atlas: Handle<TextureAtlas>,
    #[asset(path = "images", folder)]
    folder_untyped: Vec<HandleUntyped>,
    #[asset(path = "images", folder(typed))]
    folder_typed: Vec<Handle<Image>>,
    #[asset(paths("images/player.png", "images/tree.png"))]
    files_untyped: Vec<HandleUntyped>,
    #[asset(paths("images/player.png", "images/tree.png"), typed)]
    files_typed: Vec<Handle<Image>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
