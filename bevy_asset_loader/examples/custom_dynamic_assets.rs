use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RonAssetPlugin::<CustomDynamicAssetCollection>::new(&["my-assets.ron"]),
        ))
        // We need to make sure that our dynamic asset collections can be loaded from the asset file
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>()
                .register_dynamic_asset_collection::<CustomDynamicAssetCollection>()
                .with_dynamic_assets_file::<CustomDynamicAssetCollection>("custom.my-assets.ron"),
        )
        .add_systems(OnEnter(MyStates::Next), render_stuff)
        .run();
}

fn render_stuff(mut commands: Commands, assets: Res<MyAssets>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(assets.cube.clone()),
        MeshMaterial3d(assets.tree_standard_material.clone()),
        Transform::from_xyz(-1., 0., 1.),
    ));
    commands.spawn((
        Mesh3d(assets.cube.clone()),
        MeshMaterial3d(assets.player_standard_material.clone()),
        Transform::from_xyz(1., 0., 1.),
    ));
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
    // Combined image as sprite
    commands.spawn((
        Sprite {
            image: assets.combined_image.clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 200.0, 0.0),
    ));
}

#[derive(AssetCollection, Resource)]
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
    // At this point, the content of your dynamic asset file is done loading.
    // You should return untyped handles to any assets that need to finish loading for your
    // dynamic asset to be ready.
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            CustomDynamicAsset::CombinedImage {
                top_layer,
                bottom_layer,
            } => vec![
                asset_server.load_untyped(bottom_layer).untyped(),
                asset_server.load_untyped(top_layer).untyped(),
            ],
            CustomDynamicAsset::StandardMaterial {
                base_color_texture, ..
            } => vec![asset_server.load_untyped(base_color_texture).untyped()],
            CustomDynamicAsset::Cube { .. } => vec![],
        }
    }

    // This method is called when all asset handles returned from `load` are done loading.
    // The handles that you return, should also be loaded.
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            CustomDynamicAsset::CombinedImage {
                top_layer,
                bottom_layer,
            } => {
                let mut system_state =
                    SystemState::<(ResMut<Assets<Image>>, Res<AssetServer>)>::new(world);
                let (mut images, asset_server) = system_state.get_mut(world);
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
                    RenderAssetUsages::all(),
                );

                Ok(DynamicAssetType::Single(images.add(combined).untyped()))
            }
            CustomDynamicAsset::StandardMaterial {
                base_color_texture,
                base_color,
            } => {
                let mut system_state =
                    SystemState::<(ResMut<Assets<StandardMaterial>>, Res<AssetServer>)>::new(world);
                let (mut materials, asset_server) = system_state.get_mut(world);
                let color =
                    Color::linear_rgba(base_color[0], base_color[1], base_color[2], base_color[3]);
                let image = asset_server.load(base_color_texture);
                let mut material = StandardMaterial::from(color);
                material.base_color_texture = Some(image);
                material.alpha_mode = AlphaMode::Opaque;

                Ok(DynamicAssetType::Single(materials.add(material).untyped()))
            }
            CustomDynamicAsset::Cube { size } => {
                let mut meshes = world
                    .get_resource_mut::<Assets<Mesh>>()
                    .expect("Cannot get Assets<Mesh>");
                let handle = meshes
                    .add(Mesh::from(Cuboid {
                        half_size: Vec3::splat(size / 2.),
                    }))
                    .untyped();

                Ok(DynamicAssetType::Single(handle))
            }
        }
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct CustomDynamicAssetCollection(HashMap<String, CustomDynamicAsset>);

impl DynamicAssetCollection for CustomDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
