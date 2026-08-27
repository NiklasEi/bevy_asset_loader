use std::time::Duration;

use bevy::app::AppExit;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AssetPlugin, LoadContext};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn collection_handles_resolve() {
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
    )
    .add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
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
    #[asset(path = "slow.slow")]
    rendered: Handle<RasterAsset>,
}

#[allow(dead_code)]
#[derive(Resource)]
struct Resolved {
    rendered: Handle<RasterAsset>,
}

impl FromWorld for Resolved {
    fn from_world(world: &mut World) -> Self {
        let assets = world
            .get_resource::<MyAssets>()
            .expect("MyAssets not loaded");
        let loaded = world
            .get_resource::<Assets<RasterAsset>>()
            .expect("Assets<RasterAsset> not present");
        loaded
            .get(&assets.rendered)
            .expect("the collection handle must resolve in its typed store when loading completes");
        Resolved {
            rendered: assets.rendered.clone(),
        }
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
