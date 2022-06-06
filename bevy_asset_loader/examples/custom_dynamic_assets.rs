use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::{
    AssetCollection, AssetLoader, DynamicAsset, DynamicAssetCollection, DynamicAssetType,
    DynamicAssets,
};
use bevy_common_assets::ron::RonAssetPlugin;

fn main() {
    let mut app = App::new();

    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<MyAssets>()
        .build(&mut app);

    app.add_state(MyStates::CollectionLoading)
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(RonAssetPlugin::<CustomDynamicAssetCollection>::new(&[
            "assets",
        ]))
        .add_system_set(
            SystemSet::on_enter(MyStates::CollectionLoading).with_system(start_collection_loading),
        )
        .add_system_set(
            SystemSet::on_update(MyStates::CollectionLoading).with_system(check_collection_loading),
        )
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(render_stuff))
        .run();
}

fn render_stuff(mut commands: Commands, assets: Res<MyAssets>) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..PerspectiveCameraBundle::new_3d()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: assets.cube.clone(),
        material: assets.tree_standard_material.clone(),
        transform: Transform::from_xyz(-1., 0., 1.),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: assets.cube.clone(),
        material: assets.player_standard_material.clone(),
        transform: Transform::from_xyz(1., 0., 1.),
        ..default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Combined image as sprite
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        texture: assets.combined_image.clone(),
        transform: Transform::from_xyz(0.0, 200.0, 0.0),
        ..default()
    });
}

struct LoadingCollection(Handle<CustomDynamicAssetCollection>);

fn start_collection_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadingCollection(asset_server.load("custom.assets")));
}

fn check_collection_loading(
    asset_server: Res<AssetServer>,
    collection: Res<LoadingCollection>,
    mut state: ResMut<State<MyStates>>,
    mut collections: ResMut<Assets<CustomDynamicAssetCollection>>,
    mut dynamic_assets: ResMut<DynamicAssets>,
) {
    if asset_server.get_load_state(collection.0.id) == LoadState::Loaded {
        let collection = collections.remove(collection.0.clone()).unwrap();
        collection.register(&mut dynamic_assets);
        state
            .set(MyStates::AssetLoading)
            .expect("Failed to set state");
    }
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(key = "combined_image")]
    combined_image: Handle<Image>,
    #[asset(key = "tree_standard_material")]
    tree_standard_material: Handle<StandardMaterial>,
    #[asset(key = "player_standard_material")]
    player_standard_material: Handle<StandardMaterial>,
    #[asset(key = "cube")]
    cube: Handle<Mesh>,
}

#[derive(serde::Deserialize, Debug)]
enum CustomDynamicAsset {
    CombinedImage {
        bottom_layer: String,
        top_layer: String,
    },
    StandardMaterial {
        base_color: [f32; 4],
        base_color_texture: String,
    },
    Cube {
        size: f32,
    },
}

impl DynamicAsset for CustomDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped> {
        match self {
            CustomDynamicAsset::CombinedImage {
                top_layer,
                bottom_layer,
            } => vec![
                asset_server.load_untyped(bottom_layer),
                asset_server.load_untyped(top_layer),
            ],
            CustomDynamicAsset::StandardMaterial {
                base_color_texture, ..
            } => vec![asset_server.load_untyped(base_color_texture)],
            CustomDynamicAsset::Cube { .. } => vec![],
        }
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        let cell = world.cell();
        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Failed to get asset server");
        match self {
            CustomDynamicAsset::CombinedImage {
                top_layer,
                bottom_layer,
            } => {
                let mut images = cell
                    .get_resource_mut::<Assets<Image>>()
                    .expect("Failed to get image assets");
                let first = images
                    .get(asset_server.load_untyped(top_layer))
                    .expect("Failed to get first layer");
                let second = images
                    .get(asset_server.load_untyped(bottom_layer))
                    .expect("Failed to get second layer");
                let combined = Image::new(
                    second.texture_descriptor.size,
                    second.texture_descriptor.dimension,
                    second
                        .data
                        .iter()
                        .enumerate()
                        .map(|(index, data)| {
                            data.saturating_add(
                                *first
                                    .data
                                    .get(index)
                                    .expect("Images do not have the same size!"),
                            )
                        })
                        .collect(),
                    second.texture_descriptor.format,
                );

                Ok(DynamicAssetType::Single(
                    images.add(combined).clone_untyped(),
                ))
            }
            CustomDynamicAsset::StandardMaterial {
                base_color_texture,
                base_color,
            } => {
                let mut materials = cell
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .expect("Failed to get standard material assets");
                let color = Color::rgba(base_color[0], base_color[1], base_color[2], base_color[3]);
                let image = asset_server.load(base_color_texture);
                let mut material = StandardMaterial::from(color);
                material.base_color_texture = Some(image);
                material.alpha_mode = AlphaMode::Opaque;

                Ok(DynamicAssetType::Single(
                    materials.add(material).clone_untyped(),
                ))
            }
            CustomDynamicAsset::Cube { size } => {
                let mut meshes = cell
                    .get_resource_mut::<Assets<Mesh>>()
                    .expect("Failed to get mesh assets");
                let handle = meshes
                    .add(Mesh::from(shape::Cube { size: *size }))
                    .clone_untyped();

                Ok(DynamicAssetType::Single(handle))
            }
        }
    }
}

#[derive(serde::Deserialize, bevy::reflect::TypeUuid)]
#[uuid = "18dc82eb-d5f5-4d72-b0c4-e2b234367c35"]
pub struct CustomDynamicAssetCollection(HashMap<String, CustomDynamicAsset>);

impl DynamicAssetCollection for CustomDynamicAssetCollection {
    fn register(self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0 {
            dynamic_assets.register_asset(key, Box::new(asset));
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    CollectionLoading,
    AssetLoading,
    Next,
}
