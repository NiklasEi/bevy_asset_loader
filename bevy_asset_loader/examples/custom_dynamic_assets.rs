use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::{
    AssetCollection, AssetLoader, DynamicAsset, DynamicAssetCollection, DynamicAssets,
};
use bevy_common_assets::ron::RonAssetPlugin;

fn main() {
    let mut app = App::new();

    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<MyAssets>()
        .build(&mut app);

    app.add_state(MyStates::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_plugin(RonAssetPlugin::<CustomDynamicAssetCollection>::new(&[
            "assets",
        ]))
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(key = "first_combined_image")]
    first_combined_image: Handle<Image>,
    #[asset(key = "second_combined_image")]
    second_combined_image: Handle<Image>,
    #[asset(key = "first_standard_material")]
    first_standard_material: Handle<StandardMaterial>,
    #[asset(key = "second_standard_material")]
    second_standard_material: Handle<StandardMaterial>,
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
}

impl DynamicAsset for CustomDynamicAsset {
    fn load_untyped(&self, asset_server: &AssetServer) -> Vec<HandleUntyped> {
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
        }
    }

    fn single_handle(&self, world: &mut World) -> Result<HandleUntyped, ()> {
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

                Ok(images.add(combined).clone_untyped())
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

                Ok(materials.add(material).clone_untyped())
            }
        }
    }

    fn vector_of_handles(&self, _world: &mut World) -> Result<Vec<HandleUntyped>, ()> {
        Err(())
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
    AssetLoading,
    Next,
}
