use bevy::app::AppExit;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_dynamic_asset_collection_file("my.assets")
        .with_collection::<MyAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(expectations))
        .run();
}

fn expectations(
    assets: Res<MyAssets>,
    asset_server: Res<AssetServer>,
    standard_materials: Res<Assets<StandardMaterial>>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut quit: EventWriter<AppExit>,
) {
    println!("Done loading the collection. Checking expectations...");

    assert_eq!(
        asset_server.get_load_state(assets.single_file.clone()),
        LoadState::Loaded
    );
    let material = standard_materials
        .get(assets.standard_material.clone())
        .expect("Standard material should be added to its assets resource.");
    assert_eq!(
        asset_server.get_load_state(
            material
                .base_color_texture
                .clone()
                .expect("Material should have image as base color texture")
        ),
        LoadState::Loaded
    );
    let atlas = texture_atlases
        .get(assets.texture_atlas.clone())
        .expect("Texture atlas should be added to its assets resource.");
    assert_eq!(
        asset_server.get_load_state(atlas.texture.clone()),
        LoadState::Loaded
    );
    assert_eq!(assets.folder_untyped.len(), 6);
    for handle in assets.folder_untyped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }
    assert_eq!(assets.folder_typed.len(), 6);
    for handle in assets.folder_typed.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }
    assert_eq!(assets.files_untyped.len(), 2);
    for handle in assets.files_untyped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }
    assert_eq!(assets.files_typed.len(), 2);
    for handle in assets.files_typed.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }

    println!("Everything looks good!");
    println!("Quitting the application...");
    quit.send(AppExit);
}

#[allow(dead_code)]
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(key = "single_file")]
    single_file: Handle<AudioSource>,
    #[asset(key = "standard_material")]
    standard_material: Handle<StandardMaterial>,
    #[asset(key = "texture_atlas")]
    texture_atlas: Handle<TextureAtlas>,
    #[asset(key = "folder_untyped", collection)]
    folder_untyped: Vec<HandleUntyped>,
    #[asset(key = "folder_typed", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    #[asset(key = "files_untyped", collection)]
    files_untyped: Vec<HandleUntyped>,
    #[asset(key = "files_typed", collection(typed))]
    files_typed: Vec<Handle<Image>>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
