use bevy::app::AppExit;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_collection::<MyAssets>(),
        )
        .add_state(MyStates::AssetLoading)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(expectations))
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
    // Any file that can be loaded and turned into a texture atlas
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 99., columns = 8, rows = 1))]
    #[asset(path = "images/female_adventurer_sheet.png")]
    texture_atlas: Handle<TextureAtlas>,
    // Example field with type that implements `FromWorld`
    // If no derive attributes are set, `from_world` will be used to set the value.
    from_world: ColorStandardMaterial<{ u8::MAX }, 0, 0, { u8::MAX }>,

    // Load collections of assets

    // A folder (not supported on the web)
    #[asset(path = "images", collection)]
    folder_untyped: Vec<HandleUntyped>,
    // A folder loaded to typed asset handles (not supported on the web)
    #[asset(path = "images", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    // A folder loaded as map (not supported on the web)
    #[asset(path = "images", collection(mapped))]
    mapped_folder_untyped: HashMap<String, HandleUntyped>,
    // A folder loaded to typed asset handles mapped with their file names (not supported on the web)
    #[asset(path = "images", collection(typed, mapped))]
    mapped_folder_typed: HashMap<String, Handle<Image>>,
    // A collection of asset files
    #[asset(paths("images/player.png", "images/tree.png"), collection)]
    files_untyped: Vec<HandleUntyped>,
    // A collection of asset files loaded to typed asset handles
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    files_typed: Vec<Handle<Image>>,
    // A mapped collection of asset files
    #[asset(paths("images/player.png", "images/tree.png"), collection(mapped))]
    mapped_files_untyped: HashMap<String, HandleUntyped>,
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
    let material = standard_materials
        .get(&assets.from_world.handle)
        .expect("Standard material should be added to its assets resource.");
    assert_eq!(material.base_color, Color::RED);
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
    assert_eq!(assets.mapped_folder_untyped.len(), 6);
    for (name, handle) in assets.mapped_folder_untyped.iter() {
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
            &format!("images/{name}")
        );
    }
    assert_eq!(assets.mapped_folder_typed.len(), 6);
    for (name, handle) in assets.mapped_folder_typed.iter() {
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
            &format!("images/{name}")
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
    assert_eq!(assets.mapped_files_untyped.len(), 2);
    for (name, handle) in assets.mapped_files_untyped.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
        assert_eq!(
            name,
            asset_server
                .get_handle_path(handle.clone())
                .unwrap()
                .path()
                .to_str()
                .unwrap()
        );
    }
    assert_eq!(assets.mapped_files_typed.len(), 2);
    for (name, handle) in assets.mapped_files_typed.iter() {
        assert_eq!(
            asset_server.get_load_state(handle.clone()),
            LoadState::Loaded
        );
        assert_eq!(
            name,
            asset_server
                .get_handle_path(handle.clone())
                .unwrap()
                .path()
                .to_str()
                .unwrap()
        );
    }

    println!("Everything looks good!");
    println!("Quitting the application...");
    quit.send(AppExit);
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
            handle: materials.add(StandardMaterial::from(Color::rgba_u8(R, G, B, A))),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
