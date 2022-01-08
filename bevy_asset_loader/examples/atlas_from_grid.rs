use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

/// This example demonstrates how to load a texture atlas from a sprite sheet
///
/// Requires the feature 'render'
fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<MyAssets>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(draw_atlas.system()))
        .add_system_set(
            SystemSet::on_update(MyStates::Next).with_system(animate_sprite_system.system()),
        )
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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // draw the original image (whole atlas)
    let atlas = texture_atlases
        .get(my_assets.female_adventurer.clone())
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
        .insert(Timer::from_seconds(0.1, true));
}

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut Timer, &mut TextureAtlasSprite)>) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            sprite.index = (sprite.index + 1) % 8;
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
