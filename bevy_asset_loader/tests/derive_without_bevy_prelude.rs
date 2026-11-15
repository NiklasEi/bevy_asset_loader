//! The derive macro must not rely on any `bevy` items being in scope at the call site.
#![allow(dead_code)]

use bevy::asset::{Handle, UntypedHandle};
use bevy::ecs::resource::Resource;
use bevy::platform::collections::HashMap;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct Assets {
    #[asset(path = "images", collection)]
    folder: Vec<UntypedHandle>,
    #[asset(path = "images", collection(typed, mapped))]
    mapped_folder: HashMap<String, Handle<bevy::image::Image>>,
    #[asset(key = "image")]
    dynamic: Handle<bevy::image::Image>,
    #[asset(key = "image", optional)]
    optional_dynamic: Option<Handle<bevy::image::Image>>,
    #[cfg(feature = "2d")]
    #[asset(texture_atlas_layout(tile_size_x = 96, tile_size_y = 99, columns = 8, rows = 1))]
    layout: Handle<bevy::image::TextureAtlasLayout>,
    #[cfg(feature = "3d")]
    #[asset(path = "images/player.png", standard_material)]
    material: Handle<bevy::pbr::StandardMaterial>,
    #[cfg(any(feature = "2d", feature = "3d"))]
    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler(filter = nearest)))]
    image: Handle<bevy::image::Image>,
}

#[test]
fn derive_compiles_without_bevy_prelude() {}
