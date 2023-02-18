use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example shows how to use `bevy_asset_loader` without a loading state.
/// Asset collections can still be used as a convenient method to define resources containing
/// asset handles. You just need to initialise them either on the [`App`] or the [`World`].
///
/// The big difference to using a loading state is, that the here presented approach
/// does not give any guaranties about the loading status of the assets. Here, the asset
/// collections being available as resource does not automatically mean that all their asset
/// handles finished loading.
///
/// There are two asset collections in this example. On startup `ImageAssets` are initialised.
/// `AudioAssets` are initialised on the world based on user input (mouse click).
fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins)
        // Initialising the asset collection on the App:
        // The assets will start loading as soon as your application fires up.
        // The resource `ImageAssets` will be available from the beginning.
        // This requires the extension trait `AssetCollectionApp` to be in scope.
        .init_collection::<ImageAssets>()
        // This system listens for mouse clicks and then loads + inserts the AudioAssets collection
        .add_system(load_and_play_audio)
        .add_startup_system(draw)
        .run();
}

fn load_and_play_audio(world: &mut World) {
    let mouse_input = world.get_resource::<Input<MouseButton>>().unwrap();
    if mouse_input.just_pressed(MouseButton::Left) {
        // Initialize the collection on the world.
        // This will start loading the assets in this moment and directly inserts
        // the collection as resource.
        // This requires the extension trait `AssetCollectionWorld` to be in scope.
        world.init_collection::<AudioAssets>();

        let audio_assets = world.get_resource::<AudioAssets>().unwrap();
        let audio = world.get_resource::<Audio>().unwrap();
        audio.play(audio_assets.background.clone());
    }
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

fn draw(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: image_assets.player.clone(),
        transform: Transform::from_translation(Vec3::new(-150., 0., 1.)),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(150., 0., 1.)),
        ..Default::default()
    });
}
