use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[test]
fn collection_loading_simple() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
    ))
    .add_plugins(AssetLoadingPlugin)
    .insert_resource(Done(false))
    .add_systems(Startup, setup_simple)
    .add_systems(Update, (timeout, quit_when_done))
    .run();
}

fn setup_simple(mut commands: Commands) {
    commands.load_collection::<BackgroundAudio>().observe(
        |_: On<AssetCollectionLoaded<BackgroundAudio>>, mut done: ResMut<Done>| {
            done.0 = true;
        },
    );
}

#[test]
fn collection_loading_multiple_simultaneous() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
    ))
    .add_plugins(AssetLoadingPlugin)
    .insert_resource(Done(false))
    .insert_resource(LoadedCount(0u32))
    .add_systems(Startup, setup_multiple)
    .add_systems(Update, (timeout, quit_when_done))
    .run();
}

fn setup_multiple(mut commands: Commands) {
    commands
        .load_collection::<BackgroundAudio>()
        .observe(on_background_loaded);

    commands
        .load_collection::<PlopAudio>()
        .observe(on_plop_loaded);
}

fn on_background_loaded(
    _: On<AssetCollectionLoaded<BackgroundAudio>>,
    mut count: ResMut<LoadedCount>,
    mut done: ResMut<Done>,
) {
    count.0 += 1;
    if count.0 >= 2 {
        done.0 = true;
    }
}

fn on_plop_loaded(
    _: On<AssetCollectionLoaded<PlopAudio>>,
    mut count: ResMut<LoadedCount>,
    mut done: ResMut<Done>,
) {
    count.0 += 1;
    if count.0 >= 2 {
        done.0 = true;
    }
}

#[test]
fn collection_loading_sequential() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
    ))
    .add_plugins(AssetLoadingPlugin)
    .insert_resource(Done(false))
    .add_systems(Startup, setup_sequential)
    .add_systems(Update, (timeout, quit_when_done))
    .run();
}

fn setup_sequential(mut commands: Commands) {
    commands
        .load_collection::<BackgroundAudio>()
        .observe(on_first_loaded_start_second);
}

fn on_first_loaded_start_second(
    _: On<AssetCollectionLoaded<BackgroundAudio>>,
    mut commands: Commands,
    done: Res<Done>,
) {
    if !done.0 {
        commands.load_collection::<PlopAudio>().observe(
            |_: On<AssetCollectionLoaded<PlopAudio>>, mut done: ResMut<Done>| {
                done.0 = true;
            },
        );
    }
}

#[cfg(feature = "standard_dynamic_assets")]
#[test]
fn collection_loading_dynamic_assets() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
    ))
    .add_plugins(AssetLoadingPlugin)
    .insert_resource(Done(false))
    .add_systems(Startup, setup_dynamic)
    .add_systems(Update, (timeout, quit_when_done))
    .run();
}

#[cfg(feature = "standard_dynamic_assets")]
fn setup_dynamic(mut commands: Commands) {
    commands
        .load_collection::<DynamicAudio>()
        .with_dynamic_assets_file::<StandardDynamicAssetCollection>("collection_loading.assets.ron")
        .observe(
            |_: On<AssetCollectionLoaded<DynamicAudio>>,
             collection: Res<DynamicAudio>,
             mut done: ResMut<Done>| {
                let _ = &collection.background;
                done.0 = true;
            },
        );
}

#[test]
fn collection_loading_failure() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .add_plugins(AssetLoadingPlugin)
        .insert_resource(Done(false))
        .add_systems(Startup, setup_failing)
        .add_systems(Update, (timeout, quit_when_done))
        .run();
}

fn setup_failing(mut commands: Commands) {
    commands.load_collection::<NonExistentAssets>().observe(
        |_: On<AssetCollectionFailed<NonExistentAssets>>, mut done: ResMut<Done>| {
            done.0 = true;
        },
    );
}

#[derive(Resource)]
struct Done(bool);

#[derive(Resource)]
struct LoadedCount(u32);

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 60. {
        panic!("Collection loading test did not complete within 60 seconds");
    }
}

fn quit_when_done(done: Res<Done>, mut exit: MessageWriter<AppExit>) {
    if done.0 {
        exit.write(AppExit::Success);
    }
}

#[derive(AssetCollection, Resource)]
struct BackgroundAudio {
    #[asset(path = "audio/background.ogg")]
    _background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct PlopAudio {
    #[asset(path = "audio/plop.ogg")]
    _plop: Handle<AudioSource>,
}

#[cfg(feature = "standard_dynamic_assets")]
#[derive(AssetCollection, Resource)]
struct DynamicAudio {
    #[asset(key = "collection_loading.background")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct NonExistentAssets {
    #[asset(path = "audio/does_not_exist_at_all.ogg")]
    _missing: Handle<AudioSource>,
}
