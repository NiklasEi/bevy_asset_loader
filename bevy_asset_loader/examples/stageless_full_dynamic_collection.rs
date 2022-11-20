use bevy::app::AppExit;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;

fn main() {
    App::new()
        .add_loopless_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "full_dynamic_collection.assets",
                ])
                .with_collection::<MyAssets>(),
        )
        .add_enter_system(MyStates::Next, expectations)
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    // Single files loaded into a Handle<T>

    // File without post-processing
    // Type in `assets/my.assets`: `File`
    #[asset(key = "single_file")]
    single_file: Handle<AudioSource>,
    // This file will be converted to a standard material
    // The configuration for that is part of the `.assets` file
    // Type in `assets/my.assets`: `StandardMaterial`
    #[asset(key = "standard_material")]
    standard_material: Handle<StandardMaterial>,
    // This file will be converted to a texture atlas
    // The configuration for that is part of the `.assets` file
    // Type in `assets/my.assets`: `TextureAtlas`
    #[asset(key = "texture_atlas")]
    texture_atlas: Handle<TextureAtlas>,
    // Optional asset
    // The key `optional_file` is not defined in `assets/my.assets`, so the value of this field
    // will be `None`
    // Type in `assets/my.assets`: `File`, `StandardMaterial`, or `TextureAtlas`
    #[asset(key = "optional_file", optional)]
    optional_file: Option<Handle<AudioSource>>,

    // Collections of files

    // Untyped folder
    // Type in `assets/my.assets`: `Folder`
    #[asset(key = "folder_untyped", collection)]
    folder_untyped: Vec<HandleUntyped>,
    // Typed folder
    // Type in `assets/my.assets`: `Folder`
    #[asset(key = "folder_typed", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    // Untyped files
    // Type in `assets/my.assets`: `Files`
    #[asset(key = "files_untyped", collection)]
    files_untyped: Vec<HandleUntyped>,
    // Typed files
    // Type in `assets/my.assets`: `Files`
    #[asset(key = "files_typed", collection(typed))]
    files_typed: Vec<Handle<Image>>,

    // Optional file collections
    #[asset(key = "missing_key", collection, optional)]
    optional_folder_untyped: Option<Vec<HandleUntyped>>,
    #[asset(key = "missing_key", collection(typed), optional)]
    optional_folder_typed: Option<Vec<Handle<Image>>>,
    #[asset(key = "missing_key", collection, optional)]
    optional_files_untyped: Option<Vec<HandleUntyped>>,
    #[asset(key = "missing_key", collection(typed), optional)]
    optional_files_typed: Option<Vec<Handle<Image>>>,
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
        .get(&assets.standard_material)
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
        .get(&assets.texture_atlas)
        .expect("Texture atlas should be added to its assets resource.");
    assert_eq!(
        asset_server.get_load_state(atlas.texture.clone()),
        LoadState::Loaded
    );
    assert_eq!(assets.optional_file, None);
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

    assert_eq!(assets.optional_folder_untyped, None);
    assert_eq!(assets.optional_folder_typed, None);
    assert_eq!(assets.optional_files_untyped, None);
    assert_eq!(assets.optional_files_typed, None);

    println!("Everything looks good!");
    println!("Quitting the application...");
    quit.send(AppExit);
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
