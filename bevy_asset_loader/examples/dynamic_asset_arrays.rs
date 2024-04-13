use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example shows how to load an asset collection with arrays of dynamic assets `ron` file.
///
/// The assets loaded in this example are defined in `assets/dynamic_asset_arrays.asset_arrays.ron`
fn main() {
    App::new()
        .init_state::<MyStates>()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_dynamic_assets_file::<StandardDynamicAssetArrayCollection>(
                    "dynamic_asset_arrays.asset_arrays.ron",
                )
                .load_collection::<ImageAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), spawn_player_and_tree)
        .add_systems(
            Update,
            animate_sprite_system.run_if(in_state(MyStates::Next)),
        )
        .run();
}

// The keys used here are defined in `assets/dynamic_asset_arrays.assets`
#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(key = "layouts", collection(typed))]
    atlas_layout: Vec<Handle<TextureAtlasLayout>>,
    #[asset(key = "mixed", collection)]
    mixed_handlers: Vec<UntypedHandle>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn(Camera2dBundle::default());
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform {
                translation: Vec3::new(0., 150., 0.),
                ..Default::default()
            },
            texture: image_assets.mixed_handlers[1].clone().typed(),
            atlas: TextureAtlas {
                layout: image_assets.atlas_layout[0].clone(),
                index: 0,
            },
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .insert(Player);
    commands.spawn(SpriteBundle {
        texture: image_assets.mixed_handlers[1].clone().typed(),
        transform: Transform::from_translation(Vec3::new(50., 30., 1.)),
        ..Default::default()
    });
    commands.spawn(SpriteBundle {
        texture: image_assets.mixed_handlers[2].clone().typed(),
        transform: Transform::from_translation(Vec3::new(50., -90., 1.)),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(Component)]
struct Player;

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
