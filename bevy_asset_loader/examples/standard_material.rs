use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};

/// This example demonstrates how to load a standard material from a .png file
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
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
        })
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(spawn_player.system()))
        .run();
}

#[derive(AssetCollection)]
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
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        material: my_assets.player.clone(),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
