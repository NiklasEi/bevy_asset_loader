use crate::{ParseFieldError, TextureAtlasAttribute};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

#[derive(PartialEq, Debug)]
pub(crate) struct TextureAtlasAssetField {
    pub field_ident: Ident,
    pub asset_path: String,
    pub sampler: Option<SamplerType>,
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

#[derive(PartialEq, Debug)]
pub(crate) struct ImageAssetField {
    pub field_ident: Ident,
    pub asset_path: String,
    pub sampler: SamplerType,
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
    TextureAtlas(TextureAtlasAssetField),
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
                let sampler = match image.sampler {
                    SamplerType::Linear => quote!(ImageSampler::linear()),
                    SamplerType::Nearest => quote!(ImageSampler::nearest()),
                };
                let descriptor = match image.sampler {
                    SamplerType::Linear => quote!(ImageSamplerDescriptor::linear()),
                    SamplerType::Nearest => quote!(ImageSamplerDescriptor::nearest()),
                };

                quote!(#token_stream #field_ident : {
                    use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
                    let cell = world.cell();
                    let asset_server = cell.get_resource::<AssetServer>().expect("Cannot get AssetServer");
                    let mut images = cell.get_resource_mut::<Assets<Image>>().expect("Cannot get resource Assets<Image>");

                    let mut handle = asset_server.load(#asset_path);
                    let mut image = images.get_mut(&handle).expect("Only asset collection fields holding an `Image` handle can be annotated with `image`");

                    let is_different_sampler = if let ImageSampler::Descriptor(descriptor) = &image.sampler {
                        !descriptor.as_wgpu().eq(&#descriptor.as_wgpu())
                    } else {
                        false
                    };

                    if is_different_sampler {
                        let mut cloned_image = image.clone();
                        cloned_image.sampler = #sampler;
                        handle = images.add(cloned_image);
                    } else {
                        image.sampler = #sampler;
                    }

                    handle
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
                                        let path = handle.path().unwrap().path();
                                        let key = ::bevy_asset_loader::mapped::MapKey::from_path(path);
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
                                        let path = handle.path().unwrap().path();
                                        let key = ::bevy_asset_loader::mapped::MapKey::from_path(path);
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
                        .get_resource_mut::<::bevy::asset::Assets<StandardMaterial>>()
                        .expect("Cannot get resource Assets<StandardMaterial>");
                    materials.add(asset_server.load::<::bevy::render::texture::Image>(#asset_path).into())
                },)
            }
            AssetField::TextureAtlas(texture_atlas) => {
                let field_ident = texture_atlas.field_ident.clone();
                let asset_path = texture_atlas.asset_path.clone();
                let tile_size_x = texture_atlas.tile_size_x;
                let tile_size_y = texture_atlas.tile_size_y;
                let columns = texture_atlas.columns;
                let rows = texture_atlas.rows;
                let padding_x = texture_atlas.padding_x;
                let padding_y = texture_atlas.padding_y;
                let offset_x = texture_atlas.offset_x;
                let offset_y = texture_atlas.offset_y;
                let sampler_handling = texture_atlas.sampler.map(|sampler_type| {
                    let sampler = match sampler_type {
                        SamplerType::Linear => quote!(::bevy::render::texture::ImageSampler::linear()),
                        SamplerType::Nearest => quote!(::bevy::render::texture::ImageSampler::nearest()),
                    };
                    let descriptor = match sampler_type {
                        SamplerType::Linear => quote!(::bevy::render::texture::ImageSamplerDescriptor::linear()),
                        SamplerType::Nearest => quote!(::bevy::render::texture::ImageSamplerDescriptor::nearest()),
                    };

                    quote!(
                        let mut images = cell.get_resource_mut::<Assets<Image>>().expect("Cannot get resource Assets<Image>");
                        let mut image = images.get_mut(&handle).expect("Only asset collection fields holding an `Image` handle can be annotated with `image`");
                        let is_different_sampler = if let ::bevy::render::texture::ImageSampler::Descriptor(descriptor) = &image.sampler {
                            !descriptor.as_wgpu().eq(&#descriptor.as_wgpu())
                        } else {
                            false
                        };

                        if is_different_sampler {
                            let mut cloned_image = image.clone();
                            cloned_image.sampler = #sampler;
                            handle = images.add(cloned_image);
                        } else {
                            image.sampler = #sampler;
                        }
                    )
                }).unwrap_or(quote!());
                quote!(#token_stream #field_ident : {
                    let cell = world.cell();
                    let asset_server = cell
                        .get_resource::<::bevy::asset::AssetServer>()
                        .expect("Cannot get AssetServer");
                    let mut atlases = cell
                        .get_resource_mut::<::bevy::asset::Assets<TextureAtlas>>()
                        .expect("Cannot get resource Assets<TextureAtlas>");
                    let mut handle = asset_server.load(#asset_path);

                    #sampler_handling

                    atlases.add(TextureAtlas::from_grid(
                        handle,
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
                                #(folder_map.insert(#asset_paths.to_owned(), asset_server.load(#asset_paths)));*;
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
                                #(folder_map.insert(#asset_paths.to_owned(), asset_server.get_handle_untyped(#asset_paths).unwrap()));*;
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
                    let path = handle.path().unwrap().path();
                    let key = ::bevy_asset_loader::mapped::MapKey::from_path(path);
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
            AssetField::StandardMaterial(BasicAssetField { asset_path, .. })
            | AssetField::TextureAtlas(TextureAtlasAssetField { asset_path, .. })
            | AssetField::Image(ImageAssetField { asset_path, .. }) => {
                let asset_path = asset_path.clone();
                quote!(#token_stream handles.push(asset_server.load::<::bevy::render::texture::Image>(#asset_path).untyped());)
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
        if missing_fields.len() == 4 {
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
            if self.sampler.is_some() {
                return Ok(AssetField::Image(ImageAssetField {
                    field_ident: self.field_ident.unwrap(),
                    asset_path: self.asset_path.unwrap(),
                    sampler: self.sampler.unwrap(),
                }));
            }
            let asset = BasicAssetField {
                field_ident: self.field_ident.unwrap(),
                asset_path: self.asset_path.unwrap(),
            };
            if self.is_standard_material {
                return Ok(AssetField::StandardMaterial(asset));
            }
            return Ok(AssetField::Basic(asset));
        }
        if missing_fields.is_empty() {
            return Ok(AssetField::TextureAtlas(TextureAtlasAssetField {
                field_ident: self.field_ident.unwrap(),
                asset_path: self.asset_path.unwrap(),
                sampler: self.sampler,
                tile_size_x: self.tile_size_x.unwrap(),
                tile_size_y: self.tile_size_y.unwrap(),
                columns: self.columns.unwrap(),
                rows: self.rows.unwrap(),
                padding_x: self.padding_x.unwrap_or_default(),
                padding_y: self.padding_y.unwrap_or_default(),
                offset_x: self.offset_x.unwrap_or_default(),
                offset_y: self.offset_y.unwrap_or_default(),
            }));
        }
        Err(vec![ParseFieldError::MissingAttributes(missing_fields)])
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
            asset.get(0).unwrap(),
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
    fn texture_atlas() {
        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/folder".to_owned()),
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
            AssetField::TextureAtlas(TextureAtlasAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/folder".to_owned(),
                sampler: None,
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
                sampler: SamplerType::Linear
            })
        );
        assert_eq!(
            asset_nearest,
            AssetField::Image(ImageAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned(),
                sampler: SamplerType::Nearest
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
