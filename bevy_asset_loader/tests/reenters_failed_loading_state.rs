#![cfg(feature = "standard_dynamic_assets")]

use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn reenters_failed_loading_state() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AudioPlugin::default(),
        StatesPlugin,
    ));
    app.init_state::<MyStates>();
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .on_failure_continue_to_state(MyStates::Error)
            .load_collection::<Audio>(),
    )
    .add_systems(OnEnter(MyStates::Load), point_at_missing.run_if(run_once))
    .add_systems(OnEnter(MyStates::Error), recover)
    .add_systems(OnEnter(MyStates::Next), exit)
    .add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
    .run();
}

fn point_at_missing(mut dynamic_assets: ResMut<DynamicAssets>) {
    dynamic_assets.register_asset(
        "sound",
        Box::new(StandardDynamicAsset::File {
            path: "audio/does_not_exist.ogg".to_owned(),
        }),
    );
}

fn recover(
    mut dynamic_assets: ResMut<DynamicAssets>,
    mut next: ResMut<NextState<MyStates>>,
    mut recovered: Local<bool>,
) {
    assert!(
        !*recovered,
        "The failure state was not reset when re-entering the loading state"
    );
    *recovered = true;
    dynamic_assets.register_asset(
        "sound",
        Box::new(StandardDynamicAsset::File {
            path: "audio/plop.ogg".to_owned(),
        }),
    );
    next.set(MyStates::Load);
}

fn exit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 10. {
        panic!("The asset loader did not change the state in 10 seconds");
    }
}

#[derive(AssetCollection, Resource)]
struct Audio {
    #[asset(key = "sound")]
    _sound: Handle<AudioSource>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Error,
    Next,
}
