#![cfg(feature = "standard_dynamic_assets")]

use std::time::Duration;

use bevy::app::AppExit;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AssetPlugin, LoadContext};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

/// Dynamic assets and folders must load files as the asset type of the collection field,
/// even if another loader is the default for the file extension.
#[test]
fn typed_dynamic_and_folder_handles_resolve() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
    app.init_asset::<ParsedAsset>();
    app.init_asset::<RasterAsset>();
    app.init_asset_loader::<RasterLoader>();
    app.init_asset_loader::<ParsedLoader>();
    app.init_state::<MyStates>();
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .load_collection::<MyAssets>()
            .finally_init_resource::<Resolved>(),
    );
    let mut dynamic_assets = app.world_mut().resource_mut::<DynamicAssets>();
    dynamic_assets.register_asset(
        "file",
        Box::new(StandardDynamicAsset::File {
            path: "slow/single.slow".to_owned(),
        }),
    );
    dynamic_assets.register_asset(
        "files",
        Box::new(StandardDynamicAsset::Files {
            paths: vec!["slow/list.slow".to_owned(), "slow/mapped.slow".to_owned()],
        }),
    );
    dynamic_assets.register_asset(
        "folder",
        Box::new(StandardDynamicAsset::Folder {
            path: "slow".to_owned(),
        }),
    );
    dynamic_assets.register_asset(
        "optional_file",
        Box::new(StandardDynamicAsset::File {
            path: "slow/alias.slow".to_owned(),
        }),
    );
    app.add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
        .add_systems(OnEnter(MyStates::Next), expect)
        .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

fn expect(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

#[allow(dead_code)]
#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "slow", collection(typed))]
    folder: Vec<Handle<RasterAsset>>,
    #[asset(path = "slow", collection(typed, mapped))]
    mapped_folder: HashMap<String, Handle<RasterAsset>>,
    #[asset(key = "file")]
    file: Handle<RasterAsset>,
    #[asset(key = "files", collection(typed))]
    files: Vec<Handle<RasterAsset>>,
    #[asset(key = "folder", collection(typed))]
    dynamic_folder: Vec<Handle<RasterAsset>>,
    #[asset(key = "optional_file", optional)]
    optional_file: Option<Handle<RasterAsset>>,
    #[asset(key = "files", optional, collection(typed))]
    optional_files: Option<Vec<Handle<RasterAsset>>>,
}

#[derive(Resource)]
struct Resolved;

impl FromWorld for Resolved {
    fn from_world(world: &mut World) -> Self {
        let assets = world
            .get_resource::<MyAssets>()
            .expect("MyAssets not loaded");
        let loaded = world
            .get_resource::<Assets<RasterAsset>>()
            .expect("Assets<RasterAsset> not present");
        let resolves = |field: &str, handle: &Handle<RasterAsset>| {
            loaded.get(handle).unwrap_or_else(|| {
                panic!("'{field}' must resolve in its typed store when loading completes")
            });
        };

        assert_eq!(assets.folder.len(), 4);
        assets
            .folder
            .iter()
            .for_each(|handle| resolves("folder", handle));
        assert_eq!(assets.mapped_folder.len(), 4);
        assets
            .mapped_folder
            .values()
            .for_each(|handle| resolves("mapped_folder", handle));
        resolves("file", &assets.file);
        assert_eq!(assets.files.len(), 2);
        assets
            .files
            .iter()
            .for_each(|handle| resolves("files", handle));
        assert_eq!(assets.dynamic_folder.len(), 4);
        assets
            .dynamic_folder
            .iter()
            .for_each(|handle| resolves("dynamic_folder", handle));
        resolves(
            "optional_file",
            assets
                .optional_file
                .as_ref()
                .expect("optional_file is registered"),
        );
        let optional_files = assets.optional_files.as_ref().expect("files is registered");
        assert_eq!(optional_files.len(), 2);
        optional_files
            .iter()
            .for_each(|handle| resolves("optional_files", handle));
        Resolved
    }
}

#[derive(Asset, TypePath)]
struct ParsedAsset;

#[derive(Asset, TypePath)]
struct RasterAsset;

#[derive(Default, TypePath)]
struct ParsedLoader;

impl AssetLoader for ParsedLoader {
    type Asset = ParsedAsset;
    type Settings = ();
    type Error = std::io::Error;

    fn extensions(&self) -> &[&str] {
        &["slow"]
    }

    async fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        Ok(ParsedAsset)
    }
}

#[derive(Default, TypePath)]
struct RasterLoader;

impl AssetLoader for RasterLoader {
    type Asset = RasterAsset;
    type Settings = ();
    type Error = std::io::Error;

    fn extensions(&self) -> &[&str] {
        &["slow"]
    }

    async fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        std::thread::sleep(Duration::from_millis(300));
        Ok(RasterAsset)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Next,
}
