use bevy::app::AppExit;
use bevy::asset::{RecursiveDependencyLoadState, UntypedAssetId};
use bevy::prelude::*;
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), expectations)
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    // Any file that can be loaded to a Handle<T>
    #[asset(path = "audio/background.ogg")]
    single_file: Handle<AudioSource>,
    // Any file that can be loaded and turned into a standard material
    #[asset(path = "images/player.png", standard_material)]
    standard_material: Handle<StandardMaterial>,
    // Create a texture atlas layout
    #[asset(texture_atlas_layout(tile_size_x = 96, tile_size_y = 99, columns = 8, rows = 1))]
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    // Example field with type that implements `FromWorld`
    // If no derive attributes are set, `from_world` will be used to set the value.
    from_world: ColorStandardMaterial<{ u8::MAX }, 0, 0, { u8::MAX }>,

    // Image asset with sampler nearest (good for crisp pixel art)
    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler = nearest))]
    image_tree_nearest: Handle<Image>,
    // Array texture
    #[asset(path = "images/array_texture.png")]
    #[asset(image(array_texture_layers = 4))]
    array_texture: Handle<Image>,

    // Load collections of assets

    // A folder (not supported on the web)
    #[asset(path = "images", collection)]
    folder_untyped: Vec<UntypedHandle>,
    // A folder loaded to typed asset handles (not supported on the web)
    #[asset(path = "images", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    // A folder loaded as map (not supported on the web)
    #[asset(path = "images", collection(mapped))]
    mapped_folder_untyped: HashMap<String, UntypedHandle>,
    // A folder loaded to typed asset handles mapped with their file names (not supported on the web)
    #[asset(path = "images", collection(typed, mapped))]
    mapped_folder_typed: HashMap<String, Handle<Image>>,
    // A collection of asset files
    #[asset(paths("images/player.png", "images/tree.png"), collection)]
    files_untyped: Vec<UntypedHandle>,
    // A collection of asset files loaded to typed asset handles
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    files_typed: Vec<Handle<Image>>,
    // A mapped collection of asset files
    #[asset(paths("images/player.png", "images/tree.png"), collection(mapped))]
    mapped_files_untyped: HashMap<String, UntypedHandle>,
    // A mapped collection of asset files loaded to typed asset handles
    #[asset(
        paths("images/player.png", "images/tree.png"),
        collection(typed, mapped)
    )]
    mapped_files_typed: HashMap<String, Handle<Image>>,
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

    assert!(is_recursively_loaded(&assets.single_file, &asset_server));
    let material = standard_materials
        .get(&assets.standard_material)
        .expect("Standard material should be added to its assets resource.");

    assert!(is_recursively_loaded(
        &material
            .base_color_texture
            .clone()
            .expect("Material should have image as base color texture"),
        &asset_server
    ));
    texture_atlas_layouts
        .get(&assets.texture_atlas_layout)
        .expect("Texture atlas layout should be added to its assets resource.");

    let material = standard_materials
        .get(&assets.from_world.handle)
        .expect("Standard material should be added to its assets resource.");
    assert_eq!(material.base_color, Color::LinearRgba(LinearRgba::RED));

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

    let image = images
        .get(&assets.array_texture)
        .expect("Image should be added to its asset resource");
    assert_eq!(image.texture_descriptor.array_layer_count(), 4);

    assert_eq!(assets.folder_untyped.len(), 7);
    for handle in assets.folder_untyped.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
    }
    assert_eq!(assets.folder_typed.len(), 7);
    for handle in assets.folder_typed.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
    }
    assert_eq!(assets.mapped_folder_untyped.len(), 7);
    for (name, handle) in assets.mapped_folder_untyped.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    assert_eq!(assets.mapped_folder_typed.len(), 7);
    for (name, handle) in assets.mapped_folder_typed.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    assert_eq!(assets.files_untyped.len(), 2);
    for handle in assets.files_untyped.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
    }
    assert_eq!(assets.files_typed.len(), 2);
    for handle in assets.files_typed.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
    }
    assert_eq!(assets.mapped_files_untyped.len(), 2);
    for (name, handle) in assets.mapped_files_untyped.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    assert_eq!(assets.mapped_files_typed.len(), 2);
    for (name, handle) in assets.mapped_files_typed.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }

    info!("Everything looks good!");
    info!("Quitting the application...");
    quit.send(AppExit::Success);
}

struct ColorStandardMaterial<const R: u8, const G: u8, const B: u8, const A: u8> {
    pub handle: Handle<StandardMaterial>,
}

impl<const R: u8, const G: u8, const B: u8, const A: u8> FromWorld
    for ColorStandardMaterial<R, G, B, A>
{
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        ColorStandardMaterial {
            handle: materials.add(StandardMaterial::from(Color::linear_rgba(
                R as f32 / u8::MAX as f32,
                G as f32 / u8::MAX as f32,
                B as f32 / u8::MAX as f32,
                A as f32 / u8::MAX as f32,
            ))),
        }
    }
}

fn is_recursively_loaded(handle: impl Into<UntypedAssetId>, asset_server: &AssetServer) -> bool {
    asset_server
        .get_recursive_dependency_load_state(handle)
        .map(|state| state.is_loaded())
        .unwrap_or(false)
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
