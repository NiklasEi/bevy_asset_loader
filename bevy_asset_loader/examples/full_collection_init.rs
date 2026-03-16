use bevy::app::AppExit;
use bevy::asset::{AssetId, UntypedAssetId};
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example uses `init_collection` to load a full fetched asset collection
///
/// See the example counterparts `full_collection` and `full_collection_no_states`
fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AssetLoadingPlugin))
        .init_collection::<MyAssets>();

    // Grab the handles immediately after init — they should already be in the resource
    let initial_handles = {
        let assets = app.world().resource::<MyAssets>();
        InitialHandles {
            single_file: assets.single_file.id(),
            standard_material: assets.standard_material.id(),
            texture_atlas_layout: assets.texture_atlas_layout.id(),
            image_tree_nearest: assets.image_tree_nearest.id(),
            array_texture: assets.array_texture.id(),
            same_image_nearest: assets.same_image_nearest.id(),
            same_image_linear: assets.same_image_linear.id(),
        }
    };

    app.insert_resource(initial_handles)
        .add_systems(Update, expectations);
    app.run();
}

#[derive(Resource)]
struct InitialHandles {
    single_file: AssetId<AudioSource>,
    standard_material: AssetId<StandardMaterial>,
    texture_atlas_layout: AssetId<TextureAtlasLayout>,
    image_tree_nearest: AssetId<Image>,
    array_texture: AssetId<Image>,
    same_image_nearest: AssetId<Image>,
    same_image_linear: AssetId<Image>,
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
    #[asset(image(sampler(filter = nearest)))]
    image_tree_nearest: Handle<Image>,
    // Array texture
    #[asset(path = "images/array_texture.png")]
    #[asset(image(array_texture_layers = 4))]
    array_texture: Handle<Image>,

    // Two fields loading the SAME image with DIFFERENT image attributes.
    // This tests that finalize clones the source image for each field independently.
    #[asset(path = "images/player.png")]
    #[asset(image(sampler(filter = nearest)))]
    same_image_nearest: Handle<Image>,
    #[asset(path = "images/player.png")]
    #[asset(image(sampler(filter = linear)))]
    same_image_linear: Handle<Image>,
    // Also use the same path as standard_material — plain load, no image processing
    #[asset(path = "images/player.png")]
    same_image_plain: Handle<Image>,

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
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    files_typed: Vec<Handle<Image>>,
}

fn expectations(
    assets: Res<MyAssets>,
    initial_handles: Res<InitialHandles>,
    asset_server: Res<AssetServer>,
    standard_materials: Res<Assets<StandardMaterial>>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    images: Res<Assets<Image>>,
    mut quit: MessageWriter<AppExit>,
) {
    // Wait until the basic file is loaded as indicator that loading has completed
    if !asset_server
        .get_load_state(assets.single_file.id())
        .is_some_and(|s| s.is_loaded())
    {
        return;
    }

    // Also wait for the image handles from finalize to be populated
    if images.get(&assets.image_tree_nearest).is_none() {
        return;
    }

    info!("All assets loaded. Checking expectations...");

    // === Handle identity ===
    // Handles grabbed right after init_collection are the same ones that end up loaded.
    assert_eq!(
        assets.single_file.id(),
        initial_handles.single_file,
        "single_file handle should be stable"
    );
    assert_eq!(
        assets.standard_material.id(),
        initial_handles.standard_material,
        "standard_material handle should be stable"
    );
    assert_eq!(
        assets.texture_atlas_layout.id(),
        initial_handles.texture_atlas_layout,
        "texture_atlas_layout handle should be stable"
    );
    assert_eq!(
        assets.image_tree_nearest.id(),
        initial_handles.image_tree_nearest,
        "image_tree_nearest handle should be stable"
    );
    assert_eq!(
        assets.array_texture.id(),
        initial_handles.array_texture,
        "array_texture handle should be stable"
    );
    assert_eq!(
        assets.same_image_nearest.id(),
        initial_handles.same_image_nearest,
        "same_image_nearest handle should be stable"
    );
    assert_eq!(
        assets.same_image_linear.id(),
        initial_handles.same_image_linear,
        "same_image_linear handle should be stable"
    );

    // === Assets exist behind these handles ===
    assert!(
        asset_server.is_loaded_with_dependencies(assets.single_file.id()),
        "single_file should be loaded"
    );

    let material = standard_materials
        .get(&assets.standard_material)
        .expect("Standard material should exist at reserved handle");
    assert!(
        material.base_color_texture.is_some(),
        "Material should have base color texture"
    );

    texture_atlas_layouts
        .get(&assets.texture_atlas_layout)
        .expect("Texture atlas layout should exist at its handle");

    let material = standard_materials
        .get(&assets.from_world.handle)
        .expect("Standard material should be added to its assets resource.");
    assert_eq!(material.base_color, Color::LinearRgba(LinearRgba::RED));

    // === Image with nearest sampler ===
    let image = images
        .get(&assets.image_tree_nearest)
        .expect("image_tree_nearest should exist at reserved handle");
    let ImageSampler::Descriptor(descriptor) = &image.sampler else {
        panic!("image_tree_nearest sampler was not set to descriptor");
    };
    assert_eq!(
        descriptor.as_wgpu(),
        ImageSamplerDescriptor {
            label: Some("image_tree_nearest".to_string()),
            ..ImageSamplerDescriptor::nearest()
        }
        .as_wgpu()
    );

    // === Array texture ===
    let image = images
        .get(&assets.array_texture)
        .expect("array_texture should exist at reserved handle");
    assert_eq!(image.texture_descriptor.array_layer_count(), 4);

    // === Same image, different attributes ===
    // Both fields load "images/player.png" but apply different samplers.
    // Each should get its own correctly processed copy.
    let nearest = images
        .get(&assets.same_image_nearest)
        .expect("same_image_nearest should exist");
    let ImageSampler::Descriptor(nearest_desc) = &nearest.sampler else {
        panic!("same_image_nearest sampler was not set");
    };
    assert_eq!(
        nearest_desc.as_wgpu(),
        ImageSamplerDescriptor {
            label: Some("same_image_nearest".to_string()),
            ..ImageSamplerDescriptor::nearest()
        }
        .as_wgpu(),
        "same_image_nearest should have nearest sampler"
    );

    let linear = images
        .get(&assets.same_image_linear)
        .expect("same_image_linear should exist");
    let ImageSampler::Descriptor(linear_desc) = &linear.sampler else {
        panic!("same_image_linear sampler was not set");
    };
    assert_eq!(
        linear_desc.as_wgpu(),
        ImageSamplerDescriptor {
            label: Some("same_image_linear".to_string()),
            ..ImageSamplerDescriptor::linear()
        }
        .as_wgpu(),
        "same_image_linear should have linear sampler"
    );

    // These are different handles pointing to differently processed copies of the same source
    assert_ne!(
        assets.same_image_nearest.id(),
        assets.same_image_linear.id(),
        "Different image attributes should produce different handles"
    );

    // The plain image (no special attributes) should also be available
    images
        .get(&assets.same_image_plain)
        .expect("same_image_plain should exist");

    // === Folders ===
    // Folder fields start empty and are populated by finalize after loading
    assert_eq!(assets.folder_untyped.len(), 8);
    for handle in assets.folder_untyped.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
    }
    assert_eq!(assets.folder_typed.len(), 8);
    for handle in assets.folder_typed.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
    }
    assert_eq!(assets.mapped_folder_untyped.len(), 8);
    for (name, handle) in assets.mapped_folder_untyped.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }
    assert_eq!(assets.mapped_folder_typed.len(), 8);
    for (name, handle) in assets.mapped_folder_typed.iter() {
        assert!(is_recursively_loaded(handle, &asset_server));
        assert_eq!(&handle.path().unwrap().to_string(), name);
    }

    // === File collection ===
    assert_eq!(assets.files_typed.len(), 2);
    for handle in assets.files_typed.iter() {
        assert!(
            asset_server.is_loaded_with_dependencies(handle.id()),
            "File collection handle should be loaded"
        );
    }

    info!("Everything looks good!");
    info!("Quitting the application...");
    quit.write(AppExit::Success);
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
