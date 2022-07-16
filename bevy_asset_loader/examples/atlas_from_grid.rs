use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how to load a texture atlas from a sprite sheet
///
/// Requires the feature '2d'
fn main() {
    App::new()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_collection::<MyAssets>(),
        )
        .add_state(MyStates::AssetLoading)
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(draw_atlas))
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(animate_sprite_system))
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    // if the sheet would have padding, we could set that with `padding_x` and `padding_y`
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 99., columns = 8, rows = 1))]
    #[asset(path = "images/female_adventurer_sheet.png")]
    female_adventurer: Handle<TextureAtlas>,
}

fn draw_atlas(
    mut commands: Commands,
    my_assets: Res<MyAssets>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    // draw the original image (whole atlas)
    let atlas = texture_atlases
        .get(&my_assets.female_adventurer)
        .expect("Failed to find our atlas");
    commands.spawn_bundle(SpriteBundle {
        texture: atlas.texture.clone(),
        transform: Transform::from_xyz(0., -150., 0.),
        ..Default::default()
    });
    // draw single texture from sprite sheet starting at index 0
    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform {
                translation: Vec3::new(0., 150., 0.),
                ..Default::default()
            },
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: my_assets.female_adventurer.clone(),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)));
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite)>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index = (sprite.index + 1) % 8;
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
