use bevy::app::AppExit;
use bevy::asset::RecursiveDependencyLoadState;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::{Progress, ProgressCounter, ProgressPlugin, ProgressSystem};

/// This example shows how to track the loading progress of your collections using `iyes_progress`
///
/// Running it will print the current progress for every frame. The five assets from
/// the two collections will be loaded rather quickly (one/a few frames). The final task
/// completes after four seconds. At that point, `iyes_progress` will continue to the next state
/// and the app will terminate.
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // track progress during `MyStates::AssetLoading` and continue to `MyStates::Next` when progress is completed
            ProgressPlugin::new(MyStates::AssetLoading).continue_to(MyStates::Next),
            FrameTimeDiagnosticsPlugin,
        ))
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .load_collection::<TextureAssets>()
                .load_collection::<AudioAssets>(),
        )
        // gracefully quit the app when `MyStates::Next` is reached
        .add_systems(OnEnter(MyStates::Next), expect)
        .add_systems(
            Update,
            (track_fake_long_task.track_progress(), print_progress)
                .chain()
                .run_if(in_state(MyStates::AssetLoading))
                .after(LoadingStateSet(MyStates::AssetLoading)),
        )
        .run();
}

// Time in seconds to complete a custom long-running task.
// If assets are loaded earlier, the current state will not
// be changed until the 'fake long task' is completed (thanks to 'iyes_progress')
const DURATION_LONG_TASK_IN_SECS: f64 = 4.0;

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
    #[asset(path = "audio/plop.ogg")]
    plop: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
struct TextureAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
    #[asset(path = "images/female_adventurer_sheet.png")]
    female_adventurer: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 99., columns = 8, rows = 1))]
    female_adventurer_layout: Handle<TextureAtlasLayout>,
}

fn track_fake_long_task(time: Res<Time>) -> Progress {
    if time.elapsed_seconds_f64() > DURATION_LONG_TASK_IN_SECS {
        info!("Long task is completed");
        true.into()
    } else {
        false.into()
    }
}

fn expect(
    audio_assets: Res<AudioAssets>,
    texture_assets: Res<TextureAssets>,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    mut quit: EventWriter<AppExit>,
) {
    assert_eq!(
        asset_server.get_recursive_dependency_load_state(audio_assets.background.clone()),
        Some(RecursiveDependencyLoadState::Loaded)
    );
    assert_eq!(
        asset_server.get_recursive_dependency_load_state(audio_assets.plop.clone()),
        Some(RecursiveDependencyLoadState::Loaded)
    );
    texture_atlas_layouts
        .get(&texture_assets.female_adventurer_layout)
        .expect("Texture atlas should be added to its assets resource.");
    assert_eq!(
        asset_server.get_recursive_dependency_load_state(texture_assets.female_adventurer.clone()),
        Some(RecursiveDependencyLoadState::Loaded)
    );
    assert_eq!(
        asset_server.get_recursive_dependency_load_state(texture_assets.player.clone()),
        Some(RecursiveDependencyLoadState::Loaded)
    );
    assert_eq!(
        asset_server.get_recursive_dependency_load_state(texture_assets.tree.clone()),
        Some(RecursiveDependencyLoadState::Loaded)
    );
    info!("Everything looks good!");
    info!("Quitting the application...");
    quit.send(AppExit);
}

fn print_progress(
    progress: Option<Res<ProgressCounter>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    if let Some(progress) = progress.map(|counter| counter.progress()) {
        if progress.done > *last_done {
            *last_done = progress.done;
            info!(
                "[Frame {}] Changed progress: {:?}",
                diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                    .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                    .unwrap_or(0.),
                progress
            );
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
