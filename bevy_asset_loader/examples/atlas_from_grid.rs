use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how to load a texture atlas from a sprite sheet
///
/// Requires the feature '2d'
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), draw_atlas)
        .add_systems(
            Update,
            animate_sprite_system.run_if(in_state(MyStates::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    // if the sheet would have padding, you could set that with `padding_x` and `padding_y`.
    // if there would be space between the top left corner of the sheet and the first sprite, you could configure that with `offset_x` and `offset_y`
    // A texture atlas layout does not have a path as no asset file will be loaded for the layout
    #[asset(texture_atlas_layout(tile_size_x = 96, tile_size_y = 99, columns = 8, rows = 1))]
    female_adventurer_layout: Handle<TextureAtlasLayout>,
    // you can configure the sampler for the sprite sheet image
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "images/female_adventurer_sheet.png")]
    female_adventurer: Handle<Image>,
}

fn draw_atlas(mut commands: Commands, my_assets: Res<MyAssets>) {
    commands.spawn(Camera2dBundle::default());
    // draw the original image (whole sprite sheet)
    commands.spawn(SpriteBundle {
        texture: my_assets.female_adventurer.clone(),
        transform: Transform::from_xyz(0., -150., 0.),
        ..Default::default()
    });
    // draw animated sprite using the texture atlas layout
    commands.spawn((
        SpriteBundle {
            texture: my_assets.female_adventurer.clone(),
            transform: Transform::from_xyz(0., 150., 0.),
            ..Default::default()
        },
        TextureAtlas::from(my_assets.female_adventurer_layout.clone()),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_sprite_system(
    time: Res<Time>,
    mut sprites_to_animate: Query<(&mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (mut timer, mut sprite) in &mut sprites_to_animate {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index = (sprite.index + 1) % 8;
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
