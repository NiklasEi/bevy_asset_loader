use bevy::app::AppExit;
use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;

/// This example requires the `standard_dynamic_assets` feature for loading the ron file
/// and the `2d` and `3d` features for `TextureAtlas` and `StandardMaterial` dynamic assets.
/// It showcases all possible configurations for dynamic assets.

fn main() {
    App::new()
        .init_state::<MyStates>()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "full_dynamic_collection.assets.ron",
                )
                .load_collection::<MyAssets>(),
        )
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
    // This file will be converted to a standard material and
    // should be the base color texture image. The configuration
    // for that is part of the `.assets` file.
    // Type in `assets/full_dynamic_collection.assets.ron`: `StandardMaterial`
    #[asset(key = "standard_material")]
    standard_material: Handle<StandardMaterial>,
    // Configuration of a texture atlas layout that is part of the `.assets` file
    // Type in `assets/full_dynamic_collection.assets.ron`: `TextureAtlasLayout`
    #[asset(key = "texture_atlas_layout")]
    texture_atlas_layout: Handle<TextureAtlasLayout>,
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
    folder_untyped: Vec<UntypedHandle>,
    #[asset(key = "folder_untyped", collection(mapped))]
    folder_untyped_mapped: HashMap<String, UntypedHandle>,
    // Typed folder
    // Type in `assets/full_dynamic_collection.assets.ron`: `Folder`
    #[asset(key = "folder_typed", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    #[asset(key = "folder_typed", collection(typed, mapped))]
    folder_typed_mapped: HashMap<String, Handle<Image>>,
    // Untyped files
    // Type in `assets/full_dynamic_collection.assets.ron`: `Files`
    #[asset(key = "files_untyped", collection)]
    files_untyped: Vec<UntypedHandle>,
    #[asset(key = "files_untyped", collection(mapped))]
    files_untyped_mapped: HashMap<String, UntypedHandle>,
    // Typed files
    // Type in `assets/full_dynamic_collection.assets.ron`: `Files`
    #[asset(key = "files_typed", collection(typed))]
    files_typed: Vec<Handle<Image>>,
    #[asset(key = "files_typed", collection(typed, mapped))]
    files_typed_mapped: HashMap<String, Handle<Image>>,

    // Optional file collections
    #[asset(key = "missing_key", collection, optional)]
    missing_optional_folder: Option<Vec<UntypedHandle>>,
    #[asset(key = "folder_untyped", collection, optional)]
    optional_folder_untyped: Option<Vec<UntypedHandle>>,
    #[asset(key = "folder_untyped", collection(mapped), optional)]
    optional_folder_untyped_mapped: Option<HashMap<String, UntypedHandle>>,
    #[asset(key = "folder_typed", collection(typed), optional)]
    optional_folder_typed: Option<Vec<Handle<Image>>>,
    #[asset(key = "folder_typed", collection(typed, mapped), optional)]
    optional_folder_typed_mapped: Option<HashMap<String, Handle<Image>>>,

    #[asset(key = "missing_key", collection, optional)]
    missing_optional_files: Option<Vec<UntypedHandle>>,
    #[asset(key = "files_untyped", collection, optional)]
    optional_files_untyped: Option<Vec<UntypedHandle>>,
    #[asset(key = "files_untyped", collection(mapped), optional)]
    optional_files_untyped_mapped: Option<HashMap<String, UntypedHandle>>,
    #[asset(key = "files_typed", collection(typed), optional)]
    optional_files_typed: Option<Vec<Handle<Image>>>,
    #[asset(key = "files_typed", collection(typed, mapped), optional)]
    optional_files_typed_mapped: Option<HashMap<String, Handle<Image>>>,
}

fn expectations(
    assets: Res<MyAssets>,
    asset_server: Res<AssetServer>,
    standard_materials: Res<Assets<StandardMaterial>>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    images: Res<Assets<Image>>,
    mut quit: EventWriter<AppExit>,
) {
    info!("Done loading the collection. Checking expectations...");

    assert_eq!(
        asset_server.get_recursive_dependency_load_state(assets.single_file.clone()),
        Some(RecursiveDependencyLoadState::Loaded)
    );
    let material = standard_materials
        .get(&assets.standard_material)
        .expect("Standard material should be added to its assets resource.");
    assert_eq!(
        asset_server.get_recursive_dependency_load_state(
            material
                .base_color_texture
                .clone()
                .expect("Material should have image as base color texture")
        ),
        Some(RecursiveDependencyLoadState::Loaded)
    );
    texture_atlas_layouts
        .get(&assets.texture_atlas_layout)
        .expect("Texture atlas layout should be added to its assets resource.");

    assert_eq!(assets.optional_file, None);
    let image = images
        .get(&assets.image_tree_nearest)
        .expect("Image should be added to its asset resource");
    let ImageSampler::Descriptor(descriptor) = &image.sampler else {
        panic!("Descriptor was not set to non default value nearest");
    };
    assert_eq!(
        descriptor.as_wgpu(),
        ImageSamplerDescriptor::nearest().as_wgpu()
    );

    assert_eq!(assets.folder_untyped.len(), 7);
    for handle in assets.folder_untyped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    assert_eq!(assets.folder_untyped_mapped.len(), 7);
    for (name, handle) in assets.folder_untyped_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    assert_eq!(assets.folder_typed.len(), 7);
    for handle in assets.folder_typed.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    assert_eq!(assets.folder_typed_mapped.len(), 7);
    for (name, handle) in assets.folder_typed_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    assert_eq!(assets.files_untyped.len(), 2);
    for handle in assets.files_untyped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    assert_eq!(assets.files_untyped_mapped.len(), 2);
    for (name, handle) in assets.files_untyped_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    assert_eq!(assets.files_typed.len(), 2);
    for handle in assets.files_typed.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.clone()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    assert_eq!(assets.files_typed_mapped.len(), 2);
    for (name, handle) in assets.files_typed_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.clone()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }

    assert_eq!(assets.missing_optional_folder, None);
    let Some(ref optional_folder_untyped) = assets.optional_folder_untyped else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_folder_untyped.len(), 7);
    for handle in optional_folder_untyped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    let Some(ref optional_folder_untyped_mapped) = assets.optional_folder_untyped_mapped else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_folder_untyped_mapped.len(), 7);
    for (name, handle) in optional_folder_untyped_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    let Some(ref optional_folder_typed) = assets.optional_folder_typed else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_folder_typed.len(), 7);
    for handle in optional_folder_typed.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    let Some(ref optional_folder_typed_mapped) = assets.optional_folder_typed_mapped else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_folder_typed_mapped.len(), 7);
    for (name, handle) in optional_folder_typed_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }

    assert_eq!(assets.missing_optional_files, None);
    let Some(ref optional_files_untyped) = assets.optional_files_untyped else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_files_untyped.len(), 2);
    for handle in optional_files_untyped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    let Some(ref optional_files_untyped_mapped) = assets.optional_files_untyped_mapped else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_files_untyped_mapped.len(), 2);
    for (name, handle) in optional_files_untyped_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.id()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    let Some(ref optional_files_typed) = assets.optional_files_typed else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_files_typed.len(), 2);
    for handle in optional_files_typed.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.clone()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
    }
    let Some(ref optional_files_typed_mapped) = assets.optional_files_typed_mapped else {
        panic!("Optional asset not loaded")
    };
    assert_eq!(optional_files_typed_mapped.len(), 2);
    for (name, handle) in optional_files_typed_mapped.iter() {
        assert_eq!(
            asset_server.get_recursive_dependency_load_state(handle.clone()),
            Some(RecursiveDependencyLoadState::Loaded)
        );
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }

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
