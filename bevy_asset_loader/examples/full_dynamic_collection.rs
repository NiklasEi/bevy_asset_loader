use bevy::app::AppExit;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::render::texture::ImageSampler;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;
use path_slash::PathExt;

/// This example requires the `standard_dynamic_assets` feature for loading the ron file
/// and the `2d` and `3d` features for `TextureAtlas` and `StandardMaterial` dynamic assets.
/// It showcases all possible configurations for dynamic assets.

fn main() {
    App::new()
        .add_state::<MyStates>()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading).continue_to_state(MyStates::Next),
        )
        .add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            MyStates::AssetLoading,
            "full_dynamic_collection.assets.ron",
        )
        .add_collection_to_loading_state::<_, MyAssets>(MyStates::AssetLoading)
        .add_systems(Update, expectations.run_if(in_state(MyStates::Next)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    // Single files loaded into a Handle<T>

    // File without post-processing
    // Type in `assets/full_dynamic_collection.assets.ron`: `File`
    #[asset(key = "single_file")]
    single_file: Handle<AudioSource>,
    // This file will be converted to a standard material
    // The configuration for that is part of the `.assets` file
    // Type in `assets/full_dynamic_collection.assets.ron`: `StandardMaterial`
    #[asset(key = "standard_material")]
    standard_material: Handle<StandardMaterial>,
    // This file will be converted to a texture atlas
    // The configuration for that is part of the `.assets` file
    // Type in `assets/full_dynamic_collection.assets.ron`: `TextureAtlas`
    #[asset(key = "texture_atlas")]
    texture_atlas: Handle<TextureAtlas>,
    // Optional asset
    // The key `optional_file` is not defined in `assets/full_dynamic_collection.assets.ron`, so the value of this field
    // will be `None`
    // Type in `assets/full_dynamic_collection.assets.ron`: `File`, `StandardMaterial`, or `TextureAtlas`
    #[asset(key = "optional_file", optional)]
    optional_file: Option<Handle<AudioSource>>,
    // Image asset with sampler nearest (good for crisp pixel art)
    #[asset(key = "pixel_tree")]
    image_tree_nearest: Handle<Image>,

    // Collections of files

    // Untyped folder
    // Type in `assets/full_dynamic_collection.assets.ron`: `Folder`
    #[asset(key = "folder_untyped", collection)]
    folder_untyped: Vec<HandleUntyped>,
    #[asset(key = "folder_untyped", collection(mapped))]
    folder_untyped_mapped: HashMap<String, HandleUntyped>,
    // Typed folder
    // Type in `assets/full_dynamic_collection.assets.ron`: `Folder`
    #[asset(key = "folder_typed", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    #[asset(key = "folder_typed", collection(typed, mapped))]
    folder_typed_mapped: HashMap<String, Handle<Image>>,
    // Untyped files
    // Type in `assets/full_dynamic_collection.assets.ron`: `Files`
    #[asset(key = "files_untyped", collection)]
    files_untyped: Vec<HandleUntyped>,
    #[asset(key = "files_untyped", collection(mapped))]
    files_untyped_mapped: HashMap<String, HandleUntyped>,
    // Typed files
    // Type in `assets/full_dynamic_collection.assets.ron`: `Files`
    #[asset(key = "files_typed", collection(typed))]
    files_typed: Vec<Handle<Image>>,
    #[asset(key = "files_typed", collection(typed, mapped))]
    files_typed_mapped: HashMap<String, Handle<Image>>,

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
    images: Res<Assets<Image>>,
    mut quit: EventWriter<AppExit>,
) {
    info!("Done loading the collection. Checking expectations...");

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
    let image = images
        .get(&assets.image_tree_nearest)
        .expect("Image should be added to its asset resource");
    let ImageSampler::Descriptor(descriptor) = &image.sampler_descriptor else {
        panic!("Descriptor was not set to non default value nearest");
    };
    assert_eq!(descriptor, &ImageSampler::nearest_descriptor());

    assert_eq!(assets.folder_untyped.len(), 7);
    for handle in assets.folder_untyped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }
    assert_eq!(assets.folder_untyped_mapped.len(), 7);
    for (name, handle) in assets.folder_untyped_mapped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
        assert_eq!(
            &asset_server
                .get_handle_path(handle.clone())
                .unwrap()
                .path()
                .to_slash()
                .unwrap()
                .to_string(),
            name
        );
    }
    assert_eq!(assets.folder_typed.len(), 7);
    for handle in assets.folder_typed.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }
    assert_eq!(assets.folder_typed_mapped.len(), 7);
    for (name, handle) in assets.folder_typed_mapped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
        assert_eq!(
            &asset_server
                .get_handle_path(handle.clone())
                .unwrap()
                .path()
                .to_slash()
                .unwrap()
                .to_string(),
            name
        );
    }
    assert_eq!(assets.files_untyped.len(), 2);
    for handle in assets.files_untyped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }
    assert_eq!(assets.files_untyped_mapped.len(), 2);
    for (name, handle) in assets.files_untyped_mapped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
        assert_eq!(
            asset_server
                .get_handle_path(handle.clone())
                .unwrap()
                .path()
                .to_str()
                .unwrap(),
            name
        );
    }
    assert_eq!(assets.files_typed.len(), 2);
    for handle in assets.files_typed.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
    }
    assert_eq!(assets.files_typed_mapped.len(), 2);
    for (name, handle) in assets.files_typed_mapped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
        assert_eq!(
            asset_server
                .get_handle_path(handle.clone())
                .unwrap()
                .path()
                .to_str()
                .unwrap(),
            name
        );
    }

    assert_eq!(assets.optional_folder_untyped, None);
    assert_eq!(assets.optional_folder_typed, None);
    assert_eq!(assets.optional_files_untyped, None);
    assert_eq!(assets.optional_files_typed, None);

    info!("Everything looks good!");
    info!("Quitting the application...");
    quit.send(AppExit);
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
