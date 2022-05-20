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
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(spawn_player_and_tree))
        .run();
}

#[allow(dead_code)]
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(key = "single_file")]
    single_file: Handle<AudioSource>,
    #[asset(key = "standard_material")] // todo: why doesn't need `standard_material`?
    standard_material: Handle<StandardMaterial>,
    #[asset(key = "texture_atlas")] // todo: why doesn't need `texture_atlas`?
    texture_atlas: Handle<TextureAtlas>,
    #[asset(key = "folder_untyped", folder)]
    folder_untyped: Vec<HandleUntyped>,
    #[asset(key = "folder_typed", folder(typed))]
    folder_typed: Vec<Handle<Image>>,
    #[asset(key = "files_untyped", collection)]
    // Todo: `collection`? Need to separate from other field types at compile time.
    // Or do I? Can I combine more field types in the code generation like for loading? Should be able to only have one asset type per field type => Folder = Files
    files_untyped: Vec<HandleUntyped>,
    #[asset(key = "files_typed", collection(typed))]
    files_typed: Vec<Handle<Image>>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<MyAssets>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        texture: image_assets.files_typed[0].clone(),
        transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
        ..Default::default()
    });
    commands.spawn_bundle(SpriteBundle {
        texture: image_assets.files_typed[1].clone(),
        transform: Transform::from_translation(Vec3::new(50., 30., 1.)),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
