use proc_macro::TokenStream;
use crate::{ParseFieldError, TextureAtlasAttribute};
use proc_macro2::{Ident, Spacing, Span, TokenStream, Punct};
use quote::{quote, ToTokens, TokenStreamExt};

#[derive(PartialEq, Debug)]
pub(crate) struct TextureAtlasLayoutAssetField {
    pub field_ident: Ident,
    pub tile_size_x: f32,
    pub tile_size_y: f32,
    pub columns: usize,
    pub rows: usize,
    pub padding_x: f32,
    pub padding_y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SamplerType {
    Linear,
    Nearest,
}

impl TryFrom<String> for SamplerType {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "linear" => Ok(Self::Linear),
            "nearest" => Ok(Self::Nearest),
            _ => Err("Value must be either `linear` or `nearest`"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub(crate) enum ImageAddressModeType {
    #[default]
    ClampToEdge,
    Repeat,
    MirrorRepeat,
    ClampToBorder,
}

impl TryFrom<String> for ImageAddressModeType {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "clamptoedge" => Ok(Self::ClampToEdge),
            "repeat" => Ok(Self::Repeat),
            "mirrorrepeat" => Ok(Self::MirrorRepeat),
            "clamptoborder" => Ok(Self::ClampToBorder),
            _ => Err("Value must be valid ImageAddressMode"),
        }
    }
}

impl ToTokens for ImageAddressModeType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("ImageAddressMode", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        match *self {
            ImageAddressModeType::ClampToEdge => tokens.append(Ident::new("ClampToEdge", Span::call_site())),
            ImageAddressModeType::Repeat => tokens.append(Ident::new("Repeat", Span::call_site())),
            ImageAddressModeType::MirrorRepeat => tokens.append(Ident::new("MirrorRepeat", Span::call_site())),
            ImageAddressModeType::ClampToBorder => tokens.append(Ident::new("ClampToBorder", Span::call_site())),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub(crate) enum ImageFilterModeType {
    #[default]
    Nearest,
    Linear,
}

impl TryFrom<String> for ImageFilterModeType {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "linear" => Ok(Self::Linear),
            "nearest" => Ok(Self::Nearest),
            _ => Err("Value must be valid ImageFilterMode"),
        }
    }
}

impl ToTokens for ImageFilterModeType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("ImageFilterMode", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        match *self {
            ImageFilterModeType::Linear => tokens.append(Ident::new("Linear", Span::call_site())),
            ImageFilterModeType::Nearest => tokens.append(Ident::new("Nearest", Span::call_site())),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ImageCompareFunctionType {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl TryFrom<String> for ImageCompareFunctionType {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "never" => Ok(Self::Never),
            "less" => Ok(Self::Less),
            "equal" => Ok(Self::Equal),
            "lessequal" => Ok(Self::LessEqual),
            "greater" => Ok(Self::Greater),
            "notequal" => Ok(Self::NotEqual),
            "greaterequal" => Ok(Self::GreaterEqual),
            "always" => Ok(Self::Always),
            _ => Err("Value must be valid ImageCompareFunction"),
        }
    }
}

impl ToTokens for ImageCompareFunctionType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("ImageCompareFunction", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        match *self {
            ImageCompareFunctionType::Never => tokens.append(Ident::new("Never", Span::call_site())),
            ImageCompareFunctionType::Less => tokens.append(Ident::new("Less", Span::call_site())),
            ImageCompareFunctionType::Equal => tokens.append(Ident::new("Equal", Span::call_site())),
            ImageCompareFunctionType::LessEqual => tokens.append(Ident::new("LessEqual", Span::call_site())),
            ImageCompareFunctionType::Greater => tokens.append(Ident::new("Greater", Span::call_site())),
            ImageCompareFunctionType::NotEqual => tokens.append(Ident::new("NotEqual", Span::call_site())),
            ImageCompareFunctionType::GreaterEqual => tokens.append(Ident::new("GreaterEqual", Span::call_site())),
            ImageCompareFunctionType::Always => tokens.append(Ident::new("Always", Span::call_site())),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ImageSamplerBorderColorType {
    TransparentBlack,
    OpaqueBlack,
    OpaqueWhite,
    Zero,
}

impl TryFrom<String> for ImageSamplerBorderColorType {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "transparentblack" => Ok(Self::TransparentBlack),
            "opaqueblack" => Ok(Self::OpaqueBlack),
            "opaquewhite" => Ok(Self::OpaqueWhite),
            "zero" => Ok(Self::Zero),
            _ => Err("Value must be valid ImageSamplerBorderColor")
        }
    }
}

impl ToTokens for ImageSamplerBorderColorType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("ImageSamplerBorderColor", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        match *self {
            ImageSamplerBorderColorType::TransparentBlack => tokens.append(Ident::new("TransparentBlack", Span::call_site())),
            ImageSamplerBorderColorType::OpaqueBlack => tokens.append(Ident::new("OpaqueBlack", Span::call_site())),
            ImageSamplerBorderColorType::OpaqueWhite => tokens.append(Ident::new("OpaqueWhite", Span::call_site())),
            ImageSamplerBorderColorType::Zero => tokens.append(Ident::new("Zero", Span::call_site())),
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct ImageAssetField {
    pub field_ident: Ident,
    pub asset_path: String,
    pub address_mode_u: ImageAddressModeType,
    pub address_mode_v: ImageAddressModeType,
    pub address_mode_w: ImageAddressModeType,
    pub mag_filter: ImageFilterModeType,
    pub min_filter: ImageFilterModeType,
    pub mipmap_filter: ImageFilterModeType,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<ImageCompareFunctionType>,
    pub anisotropy_clamp: u16,
    pub border_color: Option<ImageSamplerBorderColorType>,
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

fn expand_to_tokens<T : ToTokens>(input: &Option<T>) -> TokenStream {
    match input {
        Some(value) => quote!(core::option::Option::Some(#value)),
        None => quote!(core::option::Option::None)
    }
}

fn image_to_settings (image: &ImageAssetField) -> TokenStream {
    let image_address_mode_u = image.address_mode_u;
    let image_address_mode_v = image.address_mode_v;
    let image_address_mode_w = image.address_mode_w;
    let image_mag_filter = image.mag_filter;
    let image_min_filter = image.min_filter;
    let image_mipmap_filter = image.mipmap_filter;
    let image_lod_min_clamp = image.lod_min_clamp;
    let image_lod_max_clamp = image.lod_max_clamp;
    let image_compare = expand_to_tokens(&image.compare);
    let image_anisotropy_clamp = image.anisotropy_clamp;
    let image_border_color = expand_to_tokens(&image.border_color);
    
    quote!(
        move |s: &mut ImageLoaderSettings| {
            let mut descriptor = ImageSamplerDescriptor::default();
            descriptor.address_mode_u = #image_address_mode_u;
            descriptor.address_mode_v = #image_address_mode_v;
            descriptor.address_mode_w = #image_address_mode_w;
            descriptor.mag_filter = #image_mag_filter;
            descriptor.min_filter = #image_min_filter;
            descriptor.mipmap_filter = #image_mipmap_filter;
            descriptor.lod_min_clamp = #image_lod_min_clamp;
            descriptor.lod_max_clamp = #image_lod_max_clamp;
            descriptor.compare = #image_compare;
            descriptor.anisotropy_clamp = #image_anisotropy_clamp;
            descriptor.border_color = #image_border_color;
            s.sampler = ImageSampler::Descriptor(descriptor);
        }
    )
}

impl AssetField {
    pub(crate) fn attach_token_stream_for_creation(
        &self,
        token_stream: TokenStream,
        name: String,
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
                let asset_path = image.asset_path.clone();
                
                let settings = image_to_settings(image);
                
                quote!(#token_stream #field_ident : {
                    use bevy::render::texture::{
                        ImageSampler, ImageSamplerDescriptor, ImageLoaderSettings, ImageAddressMode,
                        ImageFilterMode, ImageCompareFunction, ImageSamplerBorderColor,
                    };
                    let cell = world.cell();
                    let asset_server = cell.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                
                    asset_server.load_with_settings(#asset_path, #settings)
                },)
            }
            AssetField::Folder(basic, typed, mapped) => {
                let field_ident = basic.field_ident.clone();
                let field = field_ident.to_string();
                let asset_path = basic.asset_path.clone();
                match typed {
                    Typed::Yes => match mapped {
                        Mapped::No => {
                            quote!(#token_stream #field_ident : {
                                    let cell = world.cell();
                                    let asset_server = cell.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                    let folders = cell.get_resource::<::bevy::asset::Assets<::bevy::asset::LoadedFolder>>().expect("Cannot get Assets<LoadedFolder>");
                                    let handle = asset_server.get_handle(#asset_path).unwrap_or_else(|| panic!("Folders are only supported when using a loading state. Consider using 'paths' for {}.{}.", #name, #field));
                                    folders.get(handle)
                                        .unwrap()
                                        .handles
                                        .iter()
                                        .map(|handle| handle.clone().typed())
                                        .collect()
                                },)
                        }
                        Mapped::Yes => {
                            quote!(#token_stream #field_ident : {
                                    let cell = world.cell();
                                    let asset_server = cell.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                    let mut folder_map = ::bevy::utils::HashMap::default();
                                    let folders = cell.get_resource::<::bevy::asset::Assets<::bevy::asset::LoadedFolder>>().expect("Cannot get Assets<LoadedFolder>");
                                    let handle = asset_server.get_handle(#asset_path).unwrap_or_else(|| panic!("Folders are only supported when using a loading state. Consider using 'paths' for {}.{}.", #name, #field));
                                    let folder = &folders.get(handle).unwrap().handles;
                                    for handle in folder {
                                        let path = handle.path().unwrap();
                                        let key = ::bevy_asset_loader::mapped::MapKey::from_asset_path(path);
                                        folder_map.insert(key, handle.clone().typed());
                                    }
                                    folder_map
                                },)
                        }
                    },
                    Typed::No => match mapped {
                        Mapped::No => {
                            quote!(#token_stream #field_ident : {
                                    let cell = world.cell();
                                    let asset_server = cell.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                    let folders = cell.get_resource::<::bevy::asset::Assets<::bevy::asset::LoadedFolder>>().expect("Cannot get Assets<LoadedFolder>");
                                    let handle = asset_server.get_handle(#asset_path).unwrap_or_else(|| panic!("Folders are only supported when using a loading state. Consider using 'paths' for {}.{}.", #name, #field));
                                    folders.get(handle).expect("test").handles.iter().cloned().collect()
                                },)
                        }
                        Mapped::Yes => {
                            quote!(#token_stream #field_ident : {
                                    let cell = world.cell();
                                    let asset_server = cell.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                                    let mut folder_map = ::bevy::utils::HashMap::default();
                                    let folders = cell.get_resource::<::bevy::asset::Assets<::bevy::asset::LoadedFolder>>().expect("Cannot get Assets<LoadedFolder>");
                                    let handle = asset_server.get_handle(#asset_path).unwrap_or_else(|| panic!("Folders are only supported when using a loading state. Consider using 'paths' for {}.{}.", #name, #field));
                                    let folder = &folders.get(handle).unwrap().handles;
                                    for handle in folder {
                                        let path = handle.path().unwrap();
                                        let key = ::bevy_asset_loader::mapped::MapKey::from_asset_path(path);
                                        folder_map.insert(key, handle.clone());
                                    }
                                    folder_map
                                },)
                        }
                    },
                }
            }
            AssetField::StandardMaterial(basic) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                quote!(#token_stream #field_ident : {
                    let cell = world.cell();
                    let asset_server = cell.get_resource::<::bevy::asset::AssetServer>().expect("Cannot get AssetServer");
                    let mut materials = cell
                        .get_resource_mut::<::bevy::asset::Assets<::bevy::pbr::StandardMaterial>>()
                        .expect("Cannot get resource Assets<StandardMaterial>");
                    materials.add(::bevy::pbr::StandardMaterial::from(asset_server.load::<::bevy::render::texture::Image>(#asset_path)))
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
                    let cell = world.cell();
                    let mut atlases = cell
                        .get_resource_mut::<::bevy::asset::Assets<TextureAtlasLayout>>()
                        .expect("Cannot get resource Assets<TextureAtlasLayout>");
                    atlases.add(TextureAtlasLayout::from_grid(
                        Vec2::new(#tile_size_x, #tile_size_y),
                        #columns,
                        #rows,
                        Some(Vec2::new(#padding_x, #padding_y)),
                        Some(Vec2::new(#offset_x, #offset_y)),
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
                                let mut folder_map = ::bevy::utils::HashMap::default();
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
                                let mut folder_map = ::bevy::utils::HashMap::default();
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
                let asset_key = dynamic.key.clone();
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                    match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                        ::bevy_asset_loader::prelude::DynamicAssetType::Single(handle) => handle.typed(),
                        result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Single(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name)
                    }
                },)
            }
            AssetField::OptionalDynamic(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into());
                    asset.map(|asset| match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                            ::bevy_asset_loader::prelude::DynamicAssetType::Single(handle) => handle.typed(),
                            result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Single(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name)
                        }
                    )
                },)
            }
            AssetField::DynamicFileCollection(dynamic, typed, mapped) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                let load = match typed {
                    Typed::Yes => {
                        match mapped {
                            Mapped::No => quote!(match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                ::bevy_asset_loader::prelude::DynamicAssetType::Collection(mut handles) => handles.drain(..).map(|handle| handle.typed()).collect(),
                                result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Collection(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name),
                            }),
                            Mapped::Yes => {
                                let build_collection = Self::build_mapped_dynamic_file_collection(Typed::Yes, &asset_key, name);
                                quote!(match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                    #build_collection
                                })
                            }
                        }
                    }
                    Typed::No => {
                        match mapped {
                            Mapped::No =>
                                quote!(match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                    ::bevy_asset_loader::prelude::DynamicAssetType::Collection(handles) => handles,
                                    result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Collection(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name),
                                }),
                            Mapped::Yes => {
                                let build_collection = Self::build_mapped_dynamic_file_collection(Typed::No, &asset_key, name);
                                quote!(match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                    #build_collection
                                })
                            }
                        }
                    }
                };
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                    #load
                },)
            }
            AssetField::OptionalDynamicFileCollection(dynamic, typed, mapped) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                let load = match typed {
                    Typed::Yes => {
                        match mapped {
                            Mapped::No => quote!(
                                asset.map(|asset| match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                    ::bevy_asset_loader::prelude::DynamicAssetType::Collection(mut handles) => handles.drain(..).map(|handle| handle.typed()).collect(),
                                    result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Collection(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name),
                                })
                            ),
                            Mapped::Yes => {
                                let build_collection = Self::build_mapped_dynamic_file_collection(Typed::Yes, &asset_key, name);
                                quote!(
                                    asset.map(|asset| match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                        #build_collection
                                    })
                                )
                            }
                        }
                    }
                    Typed::No => {
                        match mapped {
                            Mapped::No => quote!(
                                asset.map(|asset| match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                    ::bevy_asset_loader::prelude::DynamicAssetType::Collection(handles) => handles,
                                    result => panic!("The dynamic asset '{}' cannot be created. The asset collection {} expected it to resolve to `Collection(handle)`, but {asset:?} resolves to {result:?}", #asset_key, #name),
                                })
                            ),
                            Mapped::Yes => {
                                let build_collection = Self::build_mapped_dynamic_file_collection(Typed::No, &asset_key, name);
                                quote!(
                                    asset.map(|asset| match asset.build(world).unwrap_or_else(|_| panic!("Error building the dynamic asset {:?} with the key {}", asset, #asset_key)) {
                                        #build_collection
                                    })
                                )
                            }
                        }
                    }
                };
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into());
                    #load
                },)
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
                let mut folder_map = ::bevy::utils::HashMap::default();
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
                quote!(#token_stream handles.push(asset_server.load_untyped(#asset_path).untyped());)
            }
            AssetField::Folder(asset, _, _) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream handles.push(asset_server.load_folder(#asset_path).untyped());)
            }
            AssetField::OptionalDynamic(dynamic)
            | AssetField::OptionalDynamicFileCollection(dynamic, _, _) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream {
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
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                        handles.extend(dynamic_asset.load(&asset_server));
                    }
                )
            }
            AssetField::TextureAtlasLayout(TextureAtlasLayoutAssetField { .. }) => {
                quote!(#token_stream)
            }
            AssetField::StandardMaterial(BasicAssetField { asset_path, .. }) => {
                quote!(#token_stream handles.push(asset_server.load::<::bevy::render::texture::Image>(#asset_path).untyped());)
            }
            AssetField::Image(image) => {
                let asset_path = image.asset_path.clone();
                let settings = image_to_settings(image);
                
                quote!(
                    #token_stream
                    use bevy::render::texture::{
                        ImageSampler, ImageSamplerDescriptor, ImageLoaderSettings, ImageAddressMode,
                        ImageFilterMode, ImageCompareFunction, ImageSamplerBorderColor,
                    };
                    handles.push(asset_server.load_with_settings::<::bevy::render::texture::Image, _>(#asset_path, #settings).untyped());
                )
            }
            AssetField::Files(assets, _, _) => {
                let asset_paths = assets.asset_paths.clone();
                quote!(#token_stream #(handles.push(asset_server.load_untyped(#asset_paths).untyped()));*;)
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
    
    pub tile_size_x: Option<f32>,
    pub tile_size_y: Option<f32>,
    pub columns: Option<usize>,
    pub rows: Option<usize>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
    pub offset_x: Option<f32>,
    pub offset_y: Option<f32>,
    
    pub sampler: Option<SamplerType>,
    pub address_mode_u: Option<ImageAddressModeType>,
    pub address_mode_v: Option<ImageAddressModeType>,
    pub address_mode_w: Option<ImageAddressModeType>,
    pub mag_filter: Option<ImageFilterModeType>,
    pub min_filter: Option<ImageFilterModeType>,
    pub mipmap_filter: Option<ImageFilterModeType>,
    pub lod_min_clamp: Option<f32>,
    pub lod_max_clamp: Option<f32>,
    pub compare: Option<ImageCompareFunctionType>,
    pub anisotropy_clamp: Option<u16>,
    pub border_color: Option<ImageSamplerBorderColorType>,
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
        if self.key.is_some() {
            return if self.is_optional {
                if self.is_collection {
                    Ok(AssetField::OptionalDynamicFileCollection(
                        DynamicAssetField {
                            field_ident: self.field_ident.unwrap(),
                            key: self.key.unwrap(),
                        },
                        self.is_typed.into(),
                        self.is_mapped.into(),
                    ))
                } else {
                    Ok(AssetField::OptionalDynamic(DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key: self.key.unwrap(),
                    }))
                }
            } else if self.is_collection {
                Ok(AssetField::DynamicFileCollection(
                    DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key: self.key.unwrap(),
                    },
                    self.is_typed.into(),
                    self.is_mapped.into(),
                ))
            } else {
                Ok(AssetField::Dynamic(DynamicAssetField {
                    field_ident: self.field_ident.unwrap(),
                    key: self.key.unwrap(),
                }))
            };
        }
        if self.asset_paths.is_some() {
            return Ok(AssetField::Files(
                MultipleFilesField {
                    field_ident: self.field_ident.unwrap(),
                    asset_paths: self.asset_paths.unwrap(),
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
        if self.sampler.is_some()
            || self.address_mode_u.is_some()
            || self.address_mode_v.is_some()
            || self.address_mode_w.is_some()
            || self.mag_filter.is_some()
            || self.min_filter.is_some()
            || self.mipmap_filter.is_some()
            || self.lod_min_clamp.is_some()
            || self.lod_max_clamp.is_some()
            || self.compare.is_some()
            || self.anisotropy_clamp.is_some()
            || self.border_color.is_some()
        {
            let mut mag_filter = self.mag_filter.unwrap_or_default();
            let mut min_filter = self.min_filter.unwrap_or_default();
            let mut mipmap_filter = self.mipmap_filter.unwrap_or_default();
            
            if let Some(sampler) = self.sampler {
                match sampler {
                    SamplerType::Linear => {
                        mag_filter = ImageFilterModeType::Linear;
                        min_filter = ImageFilterModeType::Linear;
                        mipmap_filter = ImageFilterModeType::Linear;
                    }
                    SamplerType::Nearest => {
                        mag_filter = ImageFilterModeType::Nearest;
                        min_filter = ImageFilterModeType::Nearest;
                        mipmap_filter = ImageFilterModeType::Nearest;
                    }
                }
            }
            
            return Ok(AssetField::Image(ImageAssetField {
                field_ident: self.field_ident.unwrap(),
                asset_path: self.asset_path.unwrap(),
                address_mode_u: self.address_mode_u.unwrap_or_default(),
                address_mode_v: self.address_mode_v.unwrap_or_default(),
                address_mode_w: self.address_mode_w.unwrap_or_default(),
                mag_filter,
                min_filter,
                mipmap_filter,
                lod_min_clamp: self.lod_min_clamp.unwrap_or(0.),
                lod_max_clamp: self.lod_max_clamp.unwrap_or(32.),
                compare: self.compare,
                anisotropy_clamp: self.anisotropy_clamp.unwrap_or(1),
                border_color: self.border_color,
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

        let asset = builder.build().expect_err("Should be pasing error");
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
            tile_size_x: Some(100.),
            tile_size_y: Some(50.),
            columns: Some(10),
            rows: Some(5),
            padding_x: Some(2.),
            offset_y: Some(3.),
            ..Default::default()
        };

        let asset = builder
            .build()
            .expect("This should be a valid TextureAtlasAsset");
        assert_eq!(
            asset,
            AssetField::TextureAtlasLayout(TextureAtlasLayoutAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                tile_size_x: 100.0,
                tile_size_y: 50.0,
                columns: 10,
                rows: 5,
                padding_x: 2.0,
                padding_y: 0.0,
                offset_x: 0.0,
                offset_y: 3.0,
            })
        );
    }

    #[test]
    fn image_asset() {
        let builder_linear = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/image.png".to_owned()),
            sampler: Some(SamplerType::Linear),
            ..Default::default()
        };

        let builder_nearest = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/image.png".to_owned()),
            sampler: Some(SamplerType::Nearest),
            ..Default::default()
        };

        let asset_linear = builder_linear
            .build()
            .expect("This should be a valid ImageAsset");
        let asset_nearest = builder_nearest
            .build()
            .expect("This should be a valid ImageAsset");

        assert_eq!(
            asset_linear,
            AssetField::Image(ImageAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned(),
                address_mode_u: Default::default(),
                address_mode_v: Default::default(),
                address_mode_w: Default::default(),
                mag_filter: ImageFilterModeType::Linear,
                min_filter: ImageFilterModeType::Linear,
                mipmap_filter: ImageFilterModeType::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 32.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            })
        );
        assert_eq!(
            asset_nearest,
            AssetField::Image(ImageAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned(),
                address_mode_u: Default::default(),
                address_mode_v: Default::default(),
                address_mode_w: Default::default(),
                mag_filter: ImageFilterModeType::Nearest,
                min_filter: ImageFilterModeType::Nearest,
                mipmap_filter: ImageFilterModeType::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 32.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
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
        builder.padding_y = Some(5.0);
        assert!(builder.build().is_err());

        let mut builder = asset_builder_dynamic();
        builder.offset_x = Some(2.5);
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
