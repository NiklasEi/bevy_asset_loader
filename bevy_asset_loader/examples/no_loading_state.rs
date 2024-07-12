use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example shows how to use a subset of `bevy_asset_loader` without a loading state.
/// Asset collections can be used as a convenient method to define resources containing
/// asset handles. They can be initialised either on the [`App`] or the [`World`].
///
/// The big difference to using a loading state is, that the here presented approach
/// does not give any guaranties about the loading status of the asset handles. Also, folders and
/// dynamic assets are not supported since they cannot instantly produce handles that will
/// eventually point to the correct loaded assets.
///
/// There are two asset collections in this example. On startup `ImageAssets` are initialised.
/// `AudioAssets` are initialised on the world based on user input (mouse click).
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Initialising the asset collection on the App:
        // The assets will start loading as soon as your application fires up.
        // The resource `ImageAssets` will be available from the beginning.
        // This requires the extension trait `AssetCollectionApp` to be in scope.
        .init_collection::<ImageAssets>()
        // This system listens for mouse clicks and then loads + inserts the AudioAssets collection
        .add_systems(Startup, draw)
        .add_systems(
            Update,
            ((load_audio, play_audio).chain(), animate_sprite_system),
        )
        .run();
}

fn load_audio(world: &mut World) {
    let mouse_input = world.get_resource::<ButtonInput<MouseButton>>().unwrap();
    if mouse_input.just_pressed(MouseButton::Left) {
        // Initialize the collection on the world.
        // This will start loading the assets at this moment and directly insert
        // the collection as a resource.
        // This requires the extension trait `AssetCollectionWorld` to be in scope.
        world.init_collection::<AudioAssets>();
    }
}

fn play_audio(audio_assets: Option<Res<AudioAssets>>, mut commands: Commands) {
    if let Some(audio_assets) = audio_assets {
        if audio_assets.is_added() {
            commands.spawn(AudioBundle {
                source: audio_assets.background.clone(),
                ..default()
            });
        }
    }
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/female_adventurer_sheet.png")]
    female_adventurer: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 96, tile_size_y = 99, columns = 8, rows = 1))]
    female_adventurer_layout: Handle<TextureAtlasLayout>,
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
    commands
        .spawn((
            SpriteBundle {
                texture: image_assets.female_adventurer.clone(),
                transform: Transform::from_translation(Vec3::new(-150., 0., 1.)),
                ..Default::default()
            },
            TextureAtlas::from(image_assets.female_adventurer_layout.clone()),
        ))
        .insert(AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )));
    commands.spawn(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(150., 0., 1.)),
        ..Default::default()
    });
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (mut timer, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index = (sprite.index + 1) % 8;
        }
    }
}
