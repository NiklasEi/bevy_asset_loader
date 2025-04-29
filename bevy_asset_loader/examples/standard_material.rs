use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

/// This example demonstrates how to load a standard material from a .png file
///
/// Requires the feature '3d'
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), spawn_player)
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(standard_material)]
    #[asset(path = "images/player.png")]
    player: Handle<StandardMaterial>,
}

fn spawn_player(
    mut commands: Commands,
    my_assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        MeshMaterial3d(my_assets.player.clone()),
        Mesh3d(meshes.add(Cuboid {
            half_size: Vec3::splat(1.0),
        })),
    ));
    commands.insert_resource(AmbientLight {
        brightness: 500.0,
        ..default()
    });
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
