//! This crate only has to compile.
#![allow(dead_code)]

type Handle<A> = ::bevy_asset::Handle<A>;
type UntypedHandle = ::bevy_asset::UntypedHandle;
type Image = ::bevy_image::Image;
type Map<V> = ::bevy_platform::collections::HashMap<String, V>;

#[derive(
    ::bevy_asset_loader::asset_collection::AssetCollection, ::bevy_ecs::resource::Resource,
)]
struct PathAssets {
    #[asset(path = "images/player.png")]
    file: Handle<Image>,
    #[asset(path = "images/player.png", standard_material)]
    standard_material: Handle<::bevy_pbr::StandardMaterial>,
    #[asset(texture_atlas_layout(tile_size_x = 96, tile_size_y = 99, columns = 8, rows = 1))]
    texture_atlas_layout: Handle<::bevy_image::TextureAtlasLayout>,
    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler(filter = nearest, wrap = repeat)))]
    image_with_sampler: Handle<Image>,
    #[asset(path = "images/array_texture.png")]
    #[asset(image(array_texture_layers = 4))]
    array_texture: Handle<Image>,
    from_world: u32,

    #[asset(path = "images", collection)]
    folder_untyped: Vec<UntypedHandle>,
    #[asset(path = "images", collection(typed))]
    folder_typed: Vec<Handle<Image>>,
    #[asset(path = "images", collection(mapped))]
    folder_untyped_mapped: Map<UntypedHandle>,
    #[asset(path = "images", collection(typed, mapped))]
    folder_typed_mapped: Map<Handle<Image>>,

    #[asset(paths("images/player.png", "images/tree.png"), collection)]
    files_untyped: Vec<UntypedHandle>,
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    files_typed: Vec<Handle<Image>>,
    #[asset(paths("images/player.png", "images/tree.png"), collection(mapped))]
    files_untyped_mapped: Map<UntypedHandle>,
    #[asset(
        paths("images/player.png", "images/tree.png"),
        collection(typed, mapped)
    )]
    files_typed_mapped: Map<Handle<Image>>,
}

#[derive(
    ::bevy_asset_loader::asset_collection::AssetCollection, ::bevy_ecs::resource::Resource,
)]
struct DynamicAssets {
    #[asset(key = "file")]
    file: Handle<Image>,
    #[asset(key = "optional_file", optional)]
    optional_file: Option<Handle<Image>>,

    #[asset(key = "collection", collection)]
    collection_untyped: Vec<UntypedHandle>,
    #[asset(key = "collection", collection(typed))]
    collection_typed: Vec<Handle<Image>>,
    #[asset(key = "collection", collection(mapped))]
    collection_untyped_mapped: Map<UntypedHandle>,
    #[asset(key = "collection", collection(typed, mapped))]
    collection_typed_mapped: Map<Handle<Image>>,

    #[asset(key = "collection", collection, optional)]
    optional_collection_untyped: Option<Vec<UntypedHandle>>,
    #[asset(key = "collection", collection(typed), optional)]
    optional_collection_typed: Option<Vec<Handle<Image>>>,
    #[asset(key = "collection", collection(mapped), optional)]
    optional_collection_untyped_mapped: Option<Map<UntypedHandle>>,
    #[asset(key = "collection", collection(typed, mapped), optional)]
    optional_collection_typed_mapped: Option<Map<Handle<Image>>>,
}
