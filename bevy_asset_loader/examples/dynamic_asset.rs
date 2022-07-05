use bevy::asset::AssetServerSettings;
use bevy::ecs::event::Events;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_asset_loader::loading_state::LoadingAssetHandles;
use bevy_asset_loader::prelude::*;
use std::marker::PhantomData;

/// This example shows how to load an asset collection with dynamic assets defined in a `.ron` file.
///
/// The assets loaded in this example are defined in `assets/dynamic_asset.assets`
fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            asset_folder: "assets".to_owned(),
            watch_for_changes: true,
        })
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<ImageAssets>()
                .with_collection::<AudioAssets>(),
        )
        .add_state(MyStates::Startup)
        .insert_resource(Msaa { samples: 1 })
        .add_system_set(
            SystemSet::on_enter(MyStates::Next)
                .with_system(spawn_player_and_tree)
                .with_system(play_background_audio),
        )
        .add_system_set(SystemSet::on_enter(MyStates::Startup).with_system(start_up))
        .add_system_set(SystemSet::on_exit(MyStates::Next).with_system(clean_up))
        .add_system_set(
            SystemSet::on_update(MyStates::Next)
                .with_system(animate_sprite_system)
                .with_system(asset_update),
        )
        .run();
}

fn asset_update(
    mut commands: Commands,
    mut updates: EventReader<AssetEvent<StandardDynamicAssetCollection>>,
    mut state: ResMut<State<MyStates>>,
) {
    for update in updates.iter() {
        info!("update {:?}", update);
        match update {
            AssetEvent::Created { .. } => {}
            AssetEvent::Modified { handle } => {
                commands.insert_resource(LoadingAssetHandles::<StandardDynamicAssetCollection> {
                    handles: vec![handle.clone_untyped()],
                    marker: PhantomData::default(),
                });
                state.set(MyStates::AssetLoading).unwrap();
            }
            AssetEvent::Removed { .. } => {}
        }
    }
}

// fn asset_update(mut world: &mut World) {
//     {
//         #[allow(clippy::type_complexity)]
//             let mut system_state: SystemState<(
//             Res<Assets<StandardDynamicAssetCollection>>,
//             ResMut<DynamicAssets>,
//             ResMut<Events<AssetEvent<StandardDynamicAssetCollection>>>
//         )> = SystemState::new(world);
//         let (
//             dynamic_asset_collections,
//             mut asset_keys,
//             mut updates
//         ) = system_state.get_mut(world);
//         if updates.is_empty() {
//             return;
//         }
//         for update in updates.drain() {
//             info!("update {:?}, got {} collections", update, dynamic_asset_collections.iter().count());
//             match update {
//                 AssetEvent::Created { handle } => {
//                     let collection = dynamic_asset_collections.get(handle).unwrap();
//                     println!("registering collection");
//                     collection.register(&mut asset_keys);
//                 }
//                 _ => {}
//             }
//         }
//     }
//     let new_collection = ImageAssets::create(world);
//     let old_tree = world.get_resource::<ImageAssets>().unwrap().tree.clone();
//     let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
//     let new_image = images.remove(new_collection.tree.clone()).unwrap();
//     let _ = images.set(old_tree, new_image);
// }

// The keys used here are defined in `assets/dynamic_asset_ron.assets`
#[derive(AssetCollection)]
struct ImageAssets {
    #[asset(key = "image.player")]
    player: Handle<TextureAtlas>,
    #[asset(key = "image.tree")]
    tree: Handle<Image>,
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(key = "sounds.background")]
    background: Handle<AudioSource>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform {
                translation: Vec3::new(0., 150., 0.),
                ..Default::default()
            },
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: image_assets.player.clone(),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)))
        .insert(Player);
    info!("tree handle {:?}", image_assets.tree.clone());
    commands.spawn_bundle(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(50., 30., 1.)),
        ..Default::default()
    });
}

fn play_background_audio(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.play(audio_assets.background.clone());
}

fn start_up(mut commands: Commands, mut state: ResMut<State<MyStates>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    state.set(MyStates::AssetLoading).unwrap();
}

fn clean_up(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    trees: Query<Entity, With<Sprite>>,
) {
    for player in players.iter() {
        commands.entity(player).despawn_recursive();
    }
    for tree in trees.iter() {
        commands.entity(tree).despawn_recursive();
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    Startup,
    AssetLoading,
    Next,
}

#[derive(Component)]
struct Player;

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
