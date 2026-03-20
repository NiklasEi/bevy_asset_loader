use crate::{ParseFieldError, TextureAtlasAttribute};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Lit, LitStr, spanned::Spanned};

#[derive(PartialEq, Debug)]
pub(crate) struct TextureAtlasLayoutAssetField {
    pub field_ident: Ident,
    pub tile_size_x: u32,
    pub tile_size_y: u32,
    pub columns: u32,
    pub rows: u32,
    pub padding_x: u32,
    pub padding_y: u32,
    pub offset_x: u32,
    pub offset_y: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum FilterType {
    Linear,
    Nearest,
}

impl TryFrom<String> for FilterType {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "linear" => Ok(Self::Linear),
            "nearest" => Ok(Self::Nearest),
            _ => Err("Value must be either `linear` or `nearest`"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum WrapMode {
    Clamp,
    Repeat,
}

impl TryFrom<String> for WrapMode {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "clamp" => Ok(Self::Clamp),
            "repeat" => Ok(Self::Repeat),
            _ => Err("Value must be either `clamp` or `repeat`"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct ImageAssetField {
    pub field_ident: Ident,
    pub asset_path: String,
    pub filter: Option<FilterType>,
    pub wrap: Option<WrapMode>,
    pub array_texture_layers: Option<u32>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct BasicAssetField {
    pub field_ident: Ident,
    pub asset_path: String,
}

#[derive(PartialEq, Debug)]
pub(crate) struct MultipleFilesField {
    pub field_ident: Ident,
    pub asset_paths: Vec<String>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct DynamicAssetField {
    pub field_ident: Ident,
    pub key: String,
}

/// Enum describing an asset field at compile-time
///
/// Variants are created from derive attributes.
#[derive(PartialEq, Debug)]
pub(crate) enum AssetField {
    Basic(BasicAssetField),
    Folder(BasicAssetField, Typed, Mapped),
    Files(MultipleFilesField, Typed, Mapped),
    TextureAtlasLayout(TextureAtlasLayoutAssetField),
    Image(ImageAssetField),
    StandardMaterial(BasicAssetField),
    Dynamic(DynamicAssetField),
    OptionalDynamic(DynamicAssetField),
    DynamicFileCollection(DynamicAssetField, Typed, Mapped),
    OptionalDynamicFileCollection(DynamicAssetField, Typed, Mapped),
}

#[derive(PartialEq, Debug)]
pub(crate) enum Typed {
    Yes,
    No,
}

impl From<bool> for Typed {
    fn from(flag: bool) -> Self {
        match flag {
            true => Typed::Yes,
            false => Typed::No,
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum Mapped {
    Yes,
    No,
}

impl From<bool> for Mapped {
    fn from(flag: bool) -> Self {
        match flag {
            true => Mapped::Yes,
            false => Mapped::No,
        }
    }
}

impl AssetField {
    pub(crate) fn attach_token_stream_for_creation(
        &self,
        token_stream: TokenStream,
    ) -> TokenStream {
        match self {
            AssetField::Basic(basic) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                quote!(#token_stream #field_ident : {
                    let asset_server = world.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                    asset_server.load(#asset_path)
                },)
            }
            AssetField::Image(image) => {
                let field_ident = image.field_ident.clone();
                quote!(#token_stream #field_ident : {
                    let images = world.resource::<::bevy::prelude::Assets<::bevy::prelude::Image>>();
                    images.reserve_handle()
                },)
            }
            AssetField::Folder(basic, _, _) => {
                let field_ident = basic.field_ident.clone();
                quote!(#token_stream #field_ident : ::std::default::Default::default(),)
            }
            AssetField::StandardMaterial(basic) => {
                let field_ident = basic.field_ident.clone();
                quote!(#token_stream #field_ident : {
                    let materials = world.resource::<::bevy::asset::Assets<::bevy::pbr::StandardMaterial>>();
                    materials.reserve_handle()
                },)
            }
            AssetField::TextureAtlasLayout(texture_atlas) => {
                let field_ident = texture_atlas.field_ident.clone();
                let tile_size_x = texture_atlas.tile_size_x;
                let tile_size_y = texture_atlas.tile_size_y;
                let columns = texture_atlas.columns;
                let rows = texture_atlas.rows;
                let padding_x = texture_atlas.padding_x;
                let padding_y = texture_atlas.padding_y;
                let offset_x = texture_atlas.offset_x;
                let offset_y = texture_atlas.offset_y;

                quote!(#token_stream #field_ident : {
                    let mut atlases = world.get_resource_mut::<::bevy::asset::Assets<::bevy::image::TextureAtlasLayout>>().expect("Cannot get Assets<TextureAtlasLayout>");
                    atlases.add(TextureAtlasLayout::from_grid(
                        ::bevy::math::UVec2::new(#tile_size_x, #tile_size_y),
                        #columns,
                        #rows,
                        Some(::bevy::math::UVec2::new(#padding_x, #padding_y)),
                        Some(::bevy::math::UVec2::new(#offset_x, #offset_y)),
                    ))
                },)
            }
            AssetField::Files(files, typed, mapped) => {
                let field_ident = files.field_ident.clone();
                let asset_paths = files.asset_paths.clone();
                match typed {
                    Typed::Yes => match mapped {
                        Mapped::No => quote!(#token_stream #field_ident : {
                                let asset_server = world.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                vec![#(asset_server.load(#asset_paths)),*]
                            },),
                        Mapped::Yes => quote!(#token_stream #field_ident : {
                                let asset_server = world.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                let mut folder_map = ::bevy::platform::collections::HashMap::default();
                                #(
                                    let path = ::bevy::asset::AssetPath::try_parse(#asset_paths.as_ref()).expect("Failed to parse asset path");
                                    let key = ::bevy_asset_loader::mapped::MapKey::from_asset_path(&path);
                                    folder_map.insert(key, asset_server.load(#asset_paths));
                                )*
                                folder_map
                            },),
                    },
                    Typed::No => match mapped {
                        Mapped::No => quote!(#token_stream #field_ident : {
                                let asset_server = world.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                vec![#(asset_server.get_handle_untyped(#asset_paths).unwrap()),*]
                            },),
                        Mapped::Yes => quote!(#token_stream #field_ident : {
                                let asset_server = world.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                let mut folder_map = ::bevy::platform::collections::HashMap::default();
                                #(
                                    let path = ::bevy::asset::AssetPath::try_parse(#asset_paths.as_ref()).expect("Failed to parse asset path");
                                    let key = ::bevy_asset_loader::mapped::MapKey::from_asset_path(&path);
                                    folder_map.insert(key, asset_server.get_handle_untyped(#asset_paths).unwrap());
                                )*
                                folder_map
                            },),
                    },
                }
            }
            AssetField::Dynamic(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                // Reserve a handle; finalize() will populate it after loading.
                // The type parameter is inferred from the struct field type.
                quote!(#token_stream #field_ident : {
                    fn __reserve<A: ::bevy::asset::Asset>(world: &::bevy::ecs::world::World) -> ::bevy::prelude::Handle<A> {
                        world.resource::<::bevy::prelude::Assets<A>>().reserve_handle()
                    }
                    __reserve(world)
                },)
            }
            AssetField::OptionalDynamic(_) | AssetField::OptionalDynamicFileCollection(_, _, _) => {
                let field_ident = self.field_ident();
                quote!(#token_stream #field_ident : None,)
            }
            AssetField::DynamicFileCollection(_, _, _) => {
                let field_ident = self.field_ident();
                quote!(#token_stream #field_ident : ::std::default::Default::default(),)
            }
        }
    }

    fn field_ident(&self) -> &Ident {
        match self {
            AssetField::Basic(b)
            | AssetField::Folder(b, _, _)
            | AssetField::StandardMaterial(b) => &b.field_ident,
            AssetField::Image(i) => &i.field_ident,
            AssetField::Files(f, _, _) => &f.field_ident,
            AssetField::TextureAtlasLayout(t) => &t.field_ident,
            AssetField::Dynamic(d)
            | AssetField::OptionalDynamic(d)
            | AssetField::DynamicFileCollection(d, _, _)
            | AssetField::OptionalDynamicFileCollection(d, _, _) => &d.field_ident,
        }
    }

    /// Generate finalize code for fields that need post-processing after assets are loaded.
    ///
    /// Fields that don't need finalization return the token stream unchanged.
    pub(crate) fn attach_token_stream_for_finalize(
        &self,
        token_stream: TokenStream,
        name: String,
    ) -> TokenStream {
        match self {
            // Basic fields don't need finalization — asset_server.load() handles are correct as-is
            AssetField::Basic(_)
            | AssetField::Files(_, _, _)
            | AssetField::TextureAtlasLayout(_) => token_stream,

            AssetField::Folder(basic, typed, mapped) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                let value = match (typed, mapped) {
                    (Typed::Yes, Mapped::No) => {
                        quote!(
                            folder
                                .handles
                                .iter()
                                .map(|handle| handle.clone().typed())
                                .collect()
                        )
                    }
                    (Typed::Yes, Mapped::Yes) => {
                        quote!({
                            let mut folder_map = ::bevy::platform::collections::HashMap::default();
                            for handle in &folder.handles {
                                let path = handle.path().unwrap();
                                let key =
                                    ::bevy_asset_loader::mapped::MapKey::from_asset_path(path);
                                folder_map.insert(key, handle.clone().typed());
                            }
                            folder_map
                        })
                    }
                    (Typed::No, Mapped::No) => {
                        quote!(folder.handles.iter().cloned().collect())
                    }
                    (Typed::No, Mapped::Yes) => {
                        quote!({
                            let mut folder_map = ::bevy::platform::collections::HashMap::default();
                            for handle in &folder.handles {
                                let path = handle.path().unwrap();
                                let key =
                                    ::bevy_asset_loader::mapped::MapKey::from_asset_path(path);
                                folder_map.insert(key, handle.clone());
                            }
                            folder_map
                        })
                    }
                };
                quote!(#token_stream {
                    let value = {
                        let mut system_state = ::bevy::ecs::system::SystemState::<(
                            Res<::bevy::asset::Assets<::bevy::asset::LoadedFolder>>,
                            Res<::bevy::prelude::AssetServer>,
                        )>::new(world);
                        let (folders, asset_server) = system_state.get(world);
                        let handle = asset_server.get_handle(#asset_path)
                            .expect("Folder handle should exist after loading");
                        let folder = folders.get(&handle)
                            .expect("LoadedFolder should be available in finalize");
                        #value
                    };
                    world.resource_mut::<Self>().#field_ident = value;
                })
            }

            AssetField::Image(image) => {
                let field_ident = image.field_ident.clone();
                let asset_path = image.asset_path.clone();
                let layers = image.array_texture_layers.unwrap_or_default();
                let filter = match image.filter {
                    Some(FilterType::Linear) | None => quote!(ImageFilterMode::Linear),
                    Some(FilterType::Nearest) => quote!(ImageFilterMode::Nearest),
                };
                let wrap = match image.wrap {
                    Some(WrapMode::Clamp) | None => quote!(ImageAddressMode::ClampToEdge),
                    Some(WrapMode::Repeat) => quote!(ImageAddressMode::Repeat),
                };
                let is_sampler_set = image.filter.is_some() || image.wrap.is_some();
                let label = Lit::Str(LitStr::new(&field_ident.to_string(), token_stream.span()));

                quote!(#token_stream {
                    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
                    let reserved = world.resource::<Self>().#field_ident.clone();
                    let mut system_state = ::bevy::ecs::system::SystemState::<(
                        ResMut<::bevy::prelude::Assets<::bevy::prelude::Image>>,
                        Res<::bevy::prelude::AssetServer>,
                    )>::new(world);
                    let (mut images, asset_server) = system_state.get_mut(world);

                    let loaded_handle = asset_server.load::<::bevy::prelude::Image>(#asset_path);
                    let loaded_image = images.get(&loaded_handle)
                        .expect("Image should be loaded before finalize is called");
                    let mut processed = loaded_image.clone();

                    if (#layers > 0) {
                        let _ = processed.reinterpret_stacked_2d_as_array(#layers);
                    }

                    let this_descriptor = ImageSamplerDescriptor {
                        label: Some(#label.to_string()),
                        address_mode_u: #wrap,
                        address_mode_v: #wrap,
                        address_mode_w: #wrap,
                        mag_filter: #filter,
                        min_filter: #filter,
                        mipmap_filter: #filter,
                        ..::std::default::Default::default()
                    };

                    if (#is_sampler_set) {
                        let is_different_sampler = if let ImageSampler::Descriptor(descriptor) = &loaded_image.sampler {
                            !descriptor.as_wgpu().eq(&this_descriptor.as_wgpu())
                        } else {
                            true
                        };

                        if is_different_sampler {
                            processed.sampler = ImageSampler::Descriptor(this_descriptor);
                        }
                    }

                    images.insert(reserved.id(), processed)
                        .expect("Reserved image handle should be valid");
                })
            }
            AssetField::StandardMaterial(basic) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                quote!(#token_stream {
                    let reserved = world.resource::<Self>().#field_ident.clone();
                    let mut system_state = ::bevy::ecs::system::SystemState::<(
                        ResMut<::bevy::asset::Assets<::bevy::pbr::StandardMaterial>>,
                        Res<::bevy::prelude::AssetServer>,
                    )>::new(world);
                    let (mut materials, asset_server) = system_state.get_mut(world);
                    let material = ::bevy::pbr::StandardMaterial::from(asset_server.load::<::bevy::image::Image>(#asset_path));
                    materials.insert(reserved.id(), material)
                        .expect("Reserved material handle should be valid");
                })
            }
            AssetField::Dynamic(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                quote!(#token_stream {
                    let reserved = world.resource::<Self>().#field_ident.clone();
                    world.resource_scope(
                        |world, asset_keys: ::bevy::prelude::Mut<::bevy_asset_loader::dynamic_asset::DynamicAssets>| {
                            let asset = asset_keys.get_asset(#asset_key.into())
                                .unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                            let built = asset.build(world)
                                .unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key));
                            match built {
                                ::bevy_asset_loader::prelude::DynamicAssetType::Single(built_handle) => {
                                    fn __move_asset<A: ::bevy::asset::Asset>(
                                        world: &mut ::bevy::ecs::world::World,
                                        from: ::bevy::prelude::UntypedHandle,
                                        to_id: ::bevy::asset::AssetId<A>,
                                    ) {
                                        let mut assets = world.resource_mut::<::bevy::prelude::Assets<A>>();
                                        let data = assets.remove(from.typed::<A>().id())
                                            .expect("Built dynamic asset should exist in Assets<T>");
                                        assets.insert(to_id, data)
                                            .expect("Reserved handle should be valid");
                                    }
                                    __move_asset(world, built_handle, reserved.id());
                                }
                                result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Single(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name)
                            }
                        }
                    );
                })
            }
            AssetField::OptionalDynamic(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                quote!(#token_stream {
                    world.resource_scope(
                        |world, asset_keys: ::bevy::prelude::Mut<::bevy_asset_loader::dynamic_asset::DynamicAssets>| {
                            let asset = asset_keys.get_asset(#asset_key.into());
                            if let Some(asset) = asset {
                                let built = asset.build(world)
                                    .unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key));
                                match built {
                                    ::bevy_asset_loader::prelude::DynamicAssetType::Single(handle) => {
                                        world.resource_mut::<Self>().#field_ident = Some(handle.typed());
                                    }
                                    result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Single(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name)
                                }
                            }
                        }
                    );
                })
            }
            AssetField::DynamicFileCollection(dynamic, typed, mapped) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                let assign =
                    Self::build_dynamic_collection_assignment(typed, mapped, &asset_key, &name);
                quote!(#token_stream {
                    world.resource_scope(
                        |world, asset_keys: ::bevy::prelude::Mut<::bevy_asset_loader::dynamic_asset::DynamicAssets>| {
                            let asset = asset_keys.get_asset(#asset_key.into())
                                .unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                            let value = {
                                let built = asset.build(world)
                                    .unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key));
                                #assign
                            };
                            world.resource_mut::<Self>().#field_ident = value;
                        }
                    );
                })
            }
            AssetField::OptionalDynamicFileCollection(dynamic, typed, mapped) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                let assign =
                    Self::build_dynamic_collection_assignment(typed, mapped, &asset_key, &name);
                quote!(#token_stream {
                    world.resource_scope(
                        |world, asset_keys: ::bevy::prelude::Mut<::bevy_asset_loader::dynamic_asset::DynamicAssets>| {
                            let asset = asset_keys.get_asset(#asset_key.into());
                            if let Some(asset) = asset {
                                let value = {
                                    let built = asset.build(world)
                                        .unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key));
                                    #assign
                                };
                                world.resource_mut::<Self>().#field_ident = Some(value);
                            }
                        }
                    );
                })
            }
        }
    }

    fn build_dynamic_collection_assignment(
        typed: &Typed,
        mapped: &Mapped,
        asset_key: &str,
        name: &str,
    ) -> TokenStream {
        match (typed, mapped) {
            (Typed::Yes, Mapped::No) => quote!(match built {
                ::bevy_asset_loader::prelude::DynamicAssetType::Collection(mut handles) =>
                    handles.drain(..).map(|handle| handle.typed()).collect(),
                result =>
                    panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Collection(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name),
            }),
            (Typed::No, Mapped::No) => quote!(match built {
                ::bevy_asset_loader::prelude::DynamicAssetType::Collection(handles) => handles,
                result =>
                    panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Collection(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name),
            }),
            (Typed::Yes, Mapped::Yes) => {
                let build_collection = Self::build_mapped_dynamic_file_collection(
                    Typed::Yes,
                    &asset_key.to_string(),
                    name.to_string(),
                );
                quote!(match built { #build_collection })
            }
            (Typed::No, Mapped::Yes) => {
                let build_collection = Self::build_mapped_dynamic_file_collection(
                    Typed::No,
                    &asset_key.to_string(),
                    name.to_string(),
                );
                quote!(match built { #build_collection })
            }
        }
    }

    fn build_mapped_dynamic_file_collection(
        typed: Typed,
        asset_key: &String,
        name: String,
    ) -> TokenStream {
        let handle = match typed {
            Typed::Yes => quote!(handle.typed()),
            Typed::No => quote!(handle),
        };
        quote!(
            ::bevy_asset_loader::prelude::DynamicAssetType::Collection(mut handles) => {
                let asset_server = world.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                let mut folder_map = ::bevy::platform::collections::HashMap::default();
                for handle in handles {
                    let path = handle.path().unwrap();
                    let key = ::bevy_asset_loader::mapped::MapKey::from_asset_path(path);
                    folder_map.insert(key, #handle);
                }
                folder_map
            },
            result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Collection(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name),
        )
    }

    pub(crate) fn attach_token_stream_for_loading(&self, token_stream: TokenStream) -> TokenStream {
        match self {
            AssetField::Basic(asset) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream {
                    let asset_server = world.get_resource::<::bevy::prelude::AssetServer>().expect("Cannot get AssetServer");
                    handles.push(asset_server.load_untyped(#asset_path).untyped());
                })
            }
            AssetField::Folder(asset, _, _) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream {
                    let asset_server = world.get_resource::<::bevy::prelude::AssetServer>().expect("Cannot get AssetServer");
                    handles.push(asset_server.load_folder(#asset_path).untyped());
                })
            }
            AssetField::OptionalDynamic(dynamic)
            | AssetField::OptionalDynamicFileCollection(dynamic, _, _) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream {
                        let mut system_state = ::bevy::ecs::system::SystemState::<(
                            Res<::bevy::prelude::AssetServer>,
                            Res<::bevy_asset_loader::prelude::DynamicAssets>,
                        )>::new(world);
                        let (asset_server, asset_keys) =
                            system_state.get(world);
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into());
                        if let Some(dynamic_asset) = dynamic_asset {
                            handles.extend(dynamic_asset.load(&asset_server));
                        }
                    }
                )
            }
            AssetField::Dynamic(dynamic) | AssetField::DynamicFileCollection(dynamic, _, _) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream {
                        let mut system_state = ::bevy::ecs::system::SystemState::<(
                            Res<::bevy::prelude::AssetServer>,
                            Res<::bevy_asset_loader::prelude::DynamicAssets>,
                        )>::new(world);
                        let (asset_server, asset_keys) =
                            system_state.get(world);
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                        handles.extend(dynamic_asset.load(&asset_server));
                    }
                )
            }
            AssetField::TextureAtlasLayout(TextureAtlasLayoutAssetField { .. }) => {
                quote!(#token_stream)
            }
            AssetField::StandardMaterial(BasicAssetField { asset_path, .. })
            | AssetField::Image(ImageAssetField { asset_path, .. }) => {
                let asset_path = asset_path.clone();
                quote!(#token_stream {
                    let asset_server = world.get_resource::<::bevy::prelude::AssetServer>().expect("Cannot get AssetServer");
                    handles.push(asset_server.load::<::bevy::image::Image>(#asset_path).untyped());
                })
            }
            AssetField::Files(assets, _, _) => {
                let asset_paths = assets.asset_paths.clone();
                quote!(#token_stream {
                    let asset_server = world.get_resource::<::bevy::prelude::AssetServer>().expect("Cannot get AssetServer");
                    #(handles.push(asset_server.load_untyped(#asset_paths).untyped()));*;
                })
            }
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct AssetBuilder {
    pub field_ident: Option<Ident>,
    pub asset_path: Option<String>,
    pub asset_paths: Option<Vec<String>>,
    pub is_standard_material: bool,
    pub is_optional: bool,
    pub is_collection: bool,
    pub is_typed: bool,
    pub is_mapped: bool,
    pub key: Option<String>,
    pub tile_size_x: Option<u32>,
    pub tile_size_y: Option<u32>,
    pub columns: Option<u32>,
    pub rows: Option<u32>,
    pub padding_x: Option<u32>,
    pub padding_y: Option<u32>,
    pub offset_x: Option<u32>,
    pub offset_y: Option<u32>,
    pub filter: Option<FilterType>,
    pub wrap: Option<WrapMode>,
    pub array_texture_layers: Option<u32>,
}

impl AssetBuilder {
    pub(crate) fn build(self) -> Result<AssetField, Vec<ParseFieldError>> {
        let mut missing_fields = vec![];
        if self.tile_size_x.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TextureAtlasAttribute::ATTRIBUTE_NAME,
                TextureAtlasAttribute::TILE_SIZE_X
            ));
        }
        if self.tile_size_y.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TextureAtlasAttribute::ATTRIBUTE_NAME,
                TextureAtlasAttribute::TILE_SIZE_Y
            ));
        }
        if self.columns.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TextureAtlasAttribute::ATTRIBUTE_NAME,
                TextureAtlasAttribute::COLUMNS
            ));
        }
        if self.rows.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TextureAtlasAttribute::ATTRIBUTE_NAME,
                TextureAtlasAttribute::ROWS
            ));
        }
        if self.asset_path.is_none() && self.asset_paths.is_none() && self.key.is_none() {
            if missing_fields.is_empty() {
                return Ok(AssetField::TextureAtlasLayout(
                    TextureAtlasLayoutAssetField {
                        field_ident: self.field_ident.unwrap(),
                        tile_size_x: self.tile_size_x.unwrap(),
                        tile_size_y: self.tile_size_y.unwrap(),
                        columns: self.columns.unwrap(),
                        rows: self.rows.unwrap(),
                        padding_x: self.padding_x.unwrap_or_default(),
                        padding_y: self.padding_y.unwrap_or_default(),
                        offset_x: self.offset_x.unwrap_or_default(),
                        offset_y: self.offset_y.unwrap_or_default(),
                    },
                ));
            } else if missing_fields.len() < 4 {
                return Err(vec![ParseFieldError::MissingAttributes(missing_fields)]);
            }
            return Err(vec![ParseFieldError::NoAttributes]);
        }
        if self.key.is_some()
            && (self.asset_path.is_some()
                || self.asset_paths.is_some()
                || missing_fields.len() < 4
                || self.padding_x.is_some()
                || self.padding_y.is_some()
                || self.offset_x.is_some()
                || self.offset_y.is_some()
                || self.is_standard_material)
        {
            return Err(vec![ParseFieldError::KeyAttributeStandsAlone]);
        }
        if self.is_optional && self.key.is_none() {
            return Err(vec![ParseFieldError::OnlyDynamicCanBeOptional]);
        }
        if self.asset_path.is_some() && self.asset_paths.is_some() {
            return Err(vec![ParseFieldError::PathAndPathsAreExclusive]);
        }
        if let Some(key) = self.key {
            return if self.is_optional {
                if self.is_collection {
                    Ok(AssetField::OptionalDynamicFileCollection(
                        DynamicAssetField {
                            field_ident: self.field_ident.unwrap(),
                            key,
                        },
                        self.is_typed.into(),
                        self.is_mapped.into(),
                    ))
                } else {
                    Ok(AssetField::OptionalDynamic(DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key,
                    }))
                }
            } else if self.is_collection {
                Ok(AssetField::DynamicFileCollection(
                    DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key,
                    },
                    self.is_typed.into(),
                    self.is_mapped.into(),
                ))
            } else {
                Ok(AssetField::Dynamic(DynamicAssetField {
                    field_ident: self.field_ident.unwrap(),
                    key,
                }))
            };
        }
        if let Some(asset_paths) = self.asset_paths {
            return Ok(AssetField::Files(
                MultipleFilesField {
                    field_ident: self.field_ident.unwrap(),
                    asset_paths,
                },
                self.is_typed.into(),
                self.is_mapped.into(),
            ));
        }
        if self.is_collection {
            return Ok(AssetField::Folder(
                BasicAssetField {
                    field_ident: self.field_ident.unwrap(),
                    asset_path: self.asset_path.unwrap(),
                },
                self.is_typed.into(),
                self.is_mapped.into(),
            ));
        }
        if self.filter.is_some() || self.array_texture_layers.is_some() {
            return Ok(AssetField::Image(ImageAssetField {
                field_ident: self.field_ident.unwrap(),
                asset_path: self.asset_path.unwrap(),
                filter: self.filter,
                wrap: self.wrap,
                array_texture_layers: self.array_texture_layers,
            }));
        }
        let asset = BasicAssetField {
            field_ident: self.field_ident.unwrap(),
            asset_path: self.asset_path.unwrap(),
        };
        if self.is_standard_material {
            return Ok(AssetField::StandardMaterial(asset));
        }

        Ok(AssetField::Basic(asset))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn basic_asset() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/image.png".to_owned()),
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            AssetField::Basic(BasicAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned()
            })
        );
    }

    #[test]
    fn standard_material() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/image.png".to_owned()),
            is_standard_material: true,
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            AssetField::StandardMaterial(BasicAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned()
            })
        );
    }

    #[test]
    fn folder() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/folder".to_owned()),
            is_collection: true,
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            AssetField::Folder(
                BasicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    asset_path: "some/folder".to_owned()
                },
                Typed::No,
                Mapped::No
            )
        );

        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/folder".to_owned()),
            is_collection: true,
            is_typed: true,
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            AssetField::Folder(
                BasicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    asset_path: "some/folder".to_owned()
                },
                Typed::Yes,
                Mapped::No
            )
        );

        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/folder".to_owned()),
            is_collection: true,
            is_mapped: true,
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            AssetField::Folder(
                BasicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    asset_path: "some/folder".to_owned()
                },
                Typed::No,
                Mapped::Yes
            )
        );

        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/folder".to_owned()),
            is_collection: true,
            is_typed: true,
            is_mapped: true,
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            AssetField::Folder(
                BasicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    asset_path: "some/folder".to_owned()
                },
                Typed::Yes,
                Mapped::Yes
            )
        );
    }

    #[test]
    fn dynamic_asset() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            key: Some("some.asset.key".to_owned()),
            ..Default::default()
        };

        let asset = builder
            .build()
            .expect("This should be a valid DynamicAsset");
        assert_eq!(
            asset,
            AssetField::Dynamic(DynamicAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                key: "some.asset.key".to_owned()
            })
        );
    }

    #[test]
    fn paths_and_path_exclusive() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some.asset".to_owned()),
            asset_paths: Some(vec!["some.asset".to_owned()]),
            ..Default::default()
        };

        let asset = builder.build().expect_err("Should be parsing error");
        assert!(variant_eq(
            asset.first().unwrap(),
            &ParseFieldError::PathAndPathsAreExclusive
        ));
    }

    #[test]
    fn multiple_files() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_paths: Some(vec!["some.asset".to_owned()]),
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid Files asset");
        assert_eq!(
            asset,
            AssetField::Files(
                MultipleFilesField {
                    field_ident: Ident::new("test", Span::call_site()),
                    asset_paths: vec!["some.asset".to_owned()]
                },
                Typed::No,
                Mapped::No
            )
        );

        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_paths: Some(vec!["some.asset".to_owned()]),
            is_typed: true,
            ..Default::default()
        };

        let asset = builder.build().expect("This should be a valid Files asset");
        assert_eq!(
            asset,
            AssetField::Files(
                MultipleFilesField {
                    field_ident: Ident::new("test", Span::call_site()),
                    asset_paths: vec!["some.asset".to_owned()]
                },
                Typed::Yes,
                Mapped::No
            )
        );
    }

    #[test]
    fn texture_atlas_layout() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            tile_size_x: Some(100),
            tile_size_y: Some(50),
            columns: Some(10),
            rows: Some(5),
            padding_x: Some(2),
            offset_y: Some(3),
            ..Default::default()
        };

        let asset = builder
            .build()
            .expect("This should be a valid TextureAtlasAsset");
        assert_eq!(
            asset,
            AssetField::TextureAtlasLayout(TextureAtlasLayoutAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                tile_size_x: 100,
                tile_size_y: 50,
                columns: 10,
                rows: 5,
                padding_x: 2,
                padding_y: 0,
                offset_x: 0,
                offset_y: 3,
            })
        );
    }

    #[test]
    fn image_asset() {
        let builder_linear = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/image.png".to_owned()),
            filter: Some(FilterType::Linear),
            wrap: None,
            ..Default::default()
        };

        let builder_nearest = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/image.png".to_owned()),
            filter: Some(FilterType::Nearest),
            wrap: None,
            ..Default::default()
        };

        let builder_layers = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/image.png".to_owned()),
            array_texture_layers: Some(42),
            ..Default::default()
        };

        let asset_linear = builder_linear
            .build()
            .expect("This should be a valid ImageAsset");
        let asset_nearest = builder_nearest
            .build()
            .expect("This should be a valid ImageAsset");
        let asset_layers = builder_layers.build().expect("Failed to build asset");

        assert_eq!(
            asset_linear,
            AssetField::Image(ImageAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned(),
                filter: Some(FilterType::Linear),
                wrap: None,
                array_texture_layers: None
            })
        );
        assert_eq!(
            asset_nearest,
            AssetField::Image(ImageAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned(),
                filter: Some(FilterType::Nearest),
                wrap: None,
                array_texture_layers: None
            })
        );
        assert_eq!(
            asset_layers,
            AssetField::Image(ImageAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned(),
                filter: None,
                wrap: None,
                array_texture_layers: Some(42)
            })
        );
    }

    #[test]
    fn dynamic_asset_does_only_accept_some_attributes() {
        let mut builder = asset_builder_dynamic();
        builder.asset_path = Some("path".to_owned());
        assert!(builder.build().is_err());

        let mut builder = asset_builder_dynamic();
        builder.is_standard_material = true;
        assert!(builder.build().is_err());

        // Required texture atlas field
        let mut builder = asset_builder_dynamic();
        builder.columns = Some(5);
        assert!(builder.build().is_err());

        // Optional texture atlas field
        let mut builder = asset_builder_dynamic();
        builder.padding_y = Some(5);
        assert!(builder.build().is_err());

        let mut builder = asset_builder_dynamic();
        builder.offset_x = Some(2);
        assert!(builder.build().is_err());

        let mut builder = asset_builder_dynamic();
        builder.is_optional = true;
        let asset = builder.build().expect("This should be a valid asset");
        assert_eq!(
            asset,
            AssetField::OptionalDynamic(DynamicAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                key: "some.asset.key".to_owned(),
            }),
            "Dynamic asset with 'optional' attribute should yield 'AssetField::OptionalDynamic'"
        );

        let mut builder = asset_builder_dynamic();
        builder.is_collection = true;
        let asset = builder.build().expect("This should be a valid asset");
        assert_eq!(
            asset,
            AssetField::DynamicFileCollection(
                DynamicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    key: "some.asset.key".to_owned(),
                },
                Typed::No,
                Mapped::No
            ),
            "Dynamic asset with 'collection' attribute should yield 'AssetField::DynamicFileCollection'"
        );

        let mut builder = asset_builder_dynamic();
        builder.is_collection = true;
        builder.is_typed = true;
        let asset = builder.build().expect("This should be a valid asset");
        assert_eq!(
            asset,
            AssetField::DynamicFileCollection(
                DynamicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    key: "some.asset.key".to_owned(),
                },
                Typed::Yes,
                Mapped::No
            ),
            "Dynamic asset with 'collection' attribute should yield 'AssetField::DynamicFileCollection'"
        );
    }

    fn asset_builder_dynamic() -> AssetBuilder {
        AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            key: Some("some.asset.key".to_owned()),
            ..AssetBuilder::default()
        }
    }

    fn variant_eq<T>(a: &T, b: &T) -> bool {
        std::mem::discriminant(a) == std::mem::discriminant(b)
    }
}
