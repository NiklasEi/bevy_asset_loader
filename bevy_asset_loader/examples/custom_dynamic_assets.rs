use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        // We need to make sure that our dynamic asset collections can be loaded from the asset file
        .add_plugin(RonAssetPlugin::<CustomDynamicAssetCollection>::new(&[
            "my-assets",
        ]))
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .with_dynamic_collections::<CustomDynamicAssetCollection>(vec!["custom.my-assets"])
                .with_collection::<MyAssets>(),
        )
        .add_state(MyStates::AssetLoading)
        .add_system_set(SystemSet::on_enter(MyStates::Next).with_system(render_stuff))
        .run();
}

fn render_stuff(mut commands: Commands, assets: Res<MyAssets>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Camera3dBundle::default()
    });
    commands.spawn(PbrBundle {
        mesh: assets.cube.clone(),
        material: assets.tree_standard_material.clone(),
        transform: Transform::from_xyz(-1., 0., 1.),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: assets.cube.clone(),
        material: assets.player_standard_material.clone(),
        transform: Transform::from_xyz(1., 0., 1.),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(Camera2dBundle {
        camera: Camera {
            priority: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    });
    // Combined image as sprite
    commands.spawn(SpriteBundle {
        texture: assets.combined_image.clone(),
        transform: Transform::from_xyz(0.0, 200.0, 0.0),
        ..default()
    });
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

#[derive(serde::Deserialize, Debug, Clone)]
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
                    .get(&asset_server.load(top_layer))
                    .expect("Failed to get first layer");
                let second = images
                    .get(&asset_server.load(bottom_layer))
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
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
