use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

fn main() {
    let mut app = App::build();
    AssetLoader::new(MyStates::AssetLoading, MyStates::Next)
        .with_collection::<SpriteSheet>()
        .init_resource::<MyTextureAtlas>()
        .build(&mut app);
    app.add_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(draw_atlas.system()))
        .add_system_set(
            SystemSet::on_update(MyStates::Next).with_system(animate_sprite_system.system()),
        )
        .run();
}

#[allow(dead_code)]
#[derive(AssetCollection)]
struct SpriteSheet {
    #[asset(path = "textures/female_adventurer.png")]
    female_adventurer: Handle<Texture>,
}

#[allow(dead_code)]
struct MyTextureAtlas {
    atlas: Handle<TextureAtlas>,
}

impl FromWorld for MyTextureAtlas {
    fn from_world(world: &mut World) -> Self {
        let cell = world.cell();
        let assets = cell
            .get_resource::<SpriteSheet>()
            .expect("SpriteSheet not loaded");
        let mut atlases = cell
            .get_resource_mut::<Assets<TextureAtlas>>()
            .expect("TextureAtlases missing");
        MyTextureAtlas {
            atlas: atlases.add(TextureAtlas::from_grid(
                assets.female_adventurer.clone(),
                Vec2::new(100., 96.),
                8,
                1,
            )),
        }
    }
}

fn draw_atlas(
    mut commands: Commands,
    texture_atlas: Res<MyTextureAtlas>,
    sprite_sheet: Res<SpriteSheet>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // draw the original texture (whole atlas)
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(sprite_sheet.female_adventurer.clone().into()),
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
            texture_atlas: texture_atlas.atlas.clone(),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true));
}

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut Timer, &mut TextureAtlasSprite)>) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            sprite.index = ((sprite.index as usize + 1) % 8) as u32;
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
