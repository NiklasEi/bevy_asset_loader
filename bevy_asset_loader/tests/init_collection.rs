use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// Test that `init_collection` creates a resource with handles that actually
/// point to loaded assets once the asset system processes them.
#[test]
fn init_collection_handles_load() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        AssetLoadingPlugin,
    ))
    .init_collection::<AudioAssets>()
    .add_systems(Update, (check_loaded, timeout));
    app.run();
}

fn check_loaded(
    audio: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
    mut exit: MessageWriter<AppExit>,
) {
    if asset_server.is_loaded_with_dependencies(audio.background.id()) {
        exit.write(AppExit::Success);
    }
}

/// Test that the handle from `init_collection` is the exact same handle
/// (same `AssetId`) as the one being tracked by the loading entity.
#[test]
fn init_collection_handle_identity() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        AssetLoadingPlugin,
    ))
    .init_collection::<AudioAssets>();

    let world = app.world_mut();
    let audio = world.resource::<AudioAssets>();
    let resource_handle_id = audio.background.id();

    // The handle from the resource should match a handle being loaded by the asset server
    let asset_server = world.resource::<AssetServer>();
    let load_state = asset_server.get_load_state(resource_handle_id);
    assert!(
        load_state.is_some(),
        "The handle from init_collection should be tracked by the AssetServer"
    );
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 30. {
        panic!("Test did not complete within 30 seconds");
    }
}
