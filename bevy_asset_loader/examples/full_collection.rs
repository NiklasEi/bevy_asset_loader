use bevy::app::AppExit;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<MyAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .insert_resource("from_world test value".to_owned())
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(expectations))
        .run();
}

#[derive(AssetCollection)]
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
    // A field that implements `FromWorld`
    from_world: FromWorldTest,

    // Load collections of assets

    // A folder (not supported on the web)
    #[asset(path = "images", collection)]
    folder_untyped: Vec<HandleUntyped>,
    // A folder loaded to typed asset handles (not supported on the web)
    #[asset(path = "images", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    // A collection of asset files
    #[asset(paths("images/player.png", "images/tree.png"), collection)]
    files_untyped: Vec<HandleUntyped>,
    // A collection of asset files loaded to typed asset handles
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    files_typed: Vec<Handle<Image>>,
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
    assert_eq!(
        assets.from_world.test_value,
        "from_world test value".to_owned()
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

struct FromWorldTest {
    test_value: String,
}

impl FromWorld for FromWorldTest {
    fn from_world(world: &mut World) -> Self {
        FromWorldTest {
            test_value: world.get_resource::<String>().unwrap().clone(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
