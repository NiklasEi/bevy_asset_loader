use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example shows how to load an asset collection with arrays of dynamic assets `ron` file.
///
/// The assets loaded in this example are defined in `assets/dynamic_asset_arrays.asset_arrays.ron`
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
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
    commands.spawn(Camera2d);
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 150., 0.)),
        Sprite::from_atlas_image(
            image_assets.mixed_handlers[1].clone().typed(),
            TextureAtlas::from(image_assets.atlas_layout[0].clone()),
        ),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player,
    ));
    commands.spawn((
        Sprite::from_image(image_assets.mixed_handlers[1].clone().typed()),
        Transform::from_translation(Vec3::new(50., 30., 1.)),
    ));
    commands.spawn((
        Sprite::from_image(image_assets.mixed_handlers[2].clone().typed()),
        Transform::from_translation(Vec3::new(50., -90., 1.)),
    ));
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

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut timer, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % 8;
            }
        }
    }
}
