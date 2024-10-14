//! This crate adds support for deriving [`AssetCollection`]
//!
//! You do not have to use it directly. Just import [`AssetCollection`] from `bevy_asset_loader`
//! and use `#[derive(AssetCollection)]` to derive the trait.

#![forbid(unsafe_code)]
#![warn(unused_imports)]

extern crate proc_macro;

mod assets;

use proc_macro::TokenStream;
use std::option::Option::Some;
use std::result::Result::{Err, Ok};

use crate::assets::*;
use proc_macro2::Ident;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;
#[cfg(any(feature = "2d", feature = "3d"))]
use syn::ExprPath;
use syn::{Data, Expr, ExprLit, Field, Fields, Index, Lit, LitStr, Meta, Token};

/// Derive macro for [`AssetCollection`]
///
/// The helper attribute ``asset`` can be used to define the path to the asset file
/// and other asset options.
#[proc_macro_derive(AssetCollection, attributes(asset))]
pub fn asset_collection_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_asset_collection(ast)
        .unwrap_or_else(to_compile_errors)
        .into()
}

pub(crate) const ASSET_ATTRIBUTE: &str = "asset";
pub(crate) const PATH_ATTRIBUTE: &str = "path";
pub(crate) const KEY_ATTRIBUTE: &str = "key";
pub(crate) const OPTIONAL_ATTRIBUTE: &str = "optional";

pub(crate) struct TextureAtlasAttribute;
impl TextureAtlasAttribute {
    pub const ATTRIBUTE_NAME_DEPRECATED: &'static str = "texture_atlas";
    pub const ATTRIBUTE_NAME: &'static str = "texture_atlas_layout";
    pub const TILE_SIZE_X: &'static str = "tile_size_x";
    pub const TILE_SIZE_Y: &'static str = "tile_size_y";
    pub const COLUMNS: &'static str = "columns";
    pub const ROWS: &'static str = "rows";
    #[allow(dead_code)]
    pub const PADDING_X: &'static str = "padding_x";
    #[allow(dead_code)]
    pub const PADDING_Y: &'static str = "padding_y";
    #[allow(dead_code)]
    pub const OFFSET_X: &'static str = "offset_x";
    #[allow(dead_code)]
    pub const OFFSET_Y: &'static str = "offset_y";
}

pub(crate) struct ImageAttribute;
impl ImageAttribute {
    pub const ATTRIBUTE_NAME: &'static str = "image";
    #[allow(dead_code)]
    pub const SAMPLER: &'static str = "sampler";
    #[allow(dead_code)]
    pub const LAYERS: &'static str = "array_texture_layers";
}

#[allow(dead_code)]
pub(crate) struct SamplerAttribute;
impl SamplerAttribute {
    #[allow(dead_code)]
    pub const FILTER: &'static str = "filter";
    #[allow(dead_code)]
    pub const CLAMP: &'static str = "clamp";
    #[allow(dead_code)]
    pub const REPEAT: &'static str = "repeat";
}

pub(crate) const COLLECTION_ATTRIBUTE: &str = "collection";
pub(crate) const PATHS_ATTRIBUTE: &str = "paths";
pub(crate) const TYPED_ATTRIBUTE: &str = "typed";
pub(crate) const MAPPED_ATTRIBUTE: &str = "mapped";
pub(crate) const STANDARD_MATERIAL_ATTRIBUTE: &str = "standard_material";

fn impl_asset_collection(
    ast: syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    let name = &ast.ident;

    let mut from_world_fields: Vec<Ident> = vec![];
    let mut assets: Vec<AssetField> = vec![];
    if let Data::Struct(ref data_struct) = ast.data {
        if let Fields::Named(ref named_fields) = data_struct.fields {
            let mut compile_errors = vec![];
            for field in named_fields.named.iter() {
                match parse_field(field) {
                    Ok(asset) => assets.push(asset),
                    Err(errors) => {
                        for error in errors {
                            match error {
                                ParseFieldError::NoAttributes => {
                                    from_world_fields.push(field.clone().ident.unwrap());
                                }
                                ParseFieldError::KeyAttributeStandsAlone => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        field.into_token_stream(),
                                        "The 'key' attribute cannot be combined with any other asset defining attributes",
                                    ));
                                }
                                ParseFieldError::OnlyDynamicCanBeOptional => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        field.into_token_stream(),
                                        "Only a dynamic asset (with 'key' attribute) can be optional",
                                    ));
                                }
                                ParseFieldError::MissingAttributes(missing_attributes) => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        field.into_token_stream(),
                                        format!(
                                            "Field is missing asset attributes: {}",
                                            missing_attributes.join(", ")
                                        ),
                                    ));
                                }
                                ParseFieldError::WrongAttributeType(token_stream, expected) => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        token_stream,
                                        format!("Wrong attribute type. Expected '{expected}'"),
                                    ));
                                }
                                ParseFieldError::UnknownAttributeType(token_stream) => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        token_stream,
                                        "Unknown attribute type",
                                    ));
                                }
                                ParseFieldError::UnknownAttribute(token_stream) => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        token_stream,
                                        "Unknown attribute",
                                    ));
                                }
                                ParseFieldError::Missing2dFeature(token_stream) => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        token_stream,
                                        "This attribute requires the '2d' feature",
                                    ));
                                }
                                ParseFieldError::Missing3dFeature(token_stream) => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        token_stream,
                                        "This attribute requires the '3d' feature",
                                    ));
                                }
                                ParseFieldError::Missing2dOr3dFeature(token_stream) => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        token_stream,
                                        "This attribute requires the '3d' or '2d' feature",
                                    ));
                                }
                                ParseFieldError::PathAndPathsAreExclusive => {
                                    compile_errors.push(syn::Error::new_spanned(
                                        field.into_token_stream(),
                                        "Either specify 'path' OR 'paths'",
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            if !compile_errors.is_empty() {
                return Err(compile_errors);
            }
        } else {
            return Err(vec![syn::Error::new_spanned(
                data_struct.fields.clone().into_token_stream(),
                "only named fields are supported to derive AssetCollection",
            )]);
        }
    } else {
        return Err(vec![syn::Error::new_spanned(
            &ast.into_token_stream(),
            "AssetCollection can only be derived for a struct",
        )]);
    }

    let asset_loading = assets.iter().fold(quote!(), |token_stream, asset| {
        asset.attach_token_stream_for_loading(token_stream)
    });
    let load_function = quote! {
            fn load(world: &mut ::bevy::ecs::world::World) -> Vec<::bevy::prelude::UntypedHandle> {
                let mut handles = vec![];
                #asset_loading
                handles
            }
    };

    let prepare_from_world = from_world_fields.iter().fold(
        quote!(),
        |es, ident| quote_spanned! {ident.span() => #es ::bevy::ecs::world::FromWorld::from_world(world),},
    );

    let mut asset_creation = assets.iter().fold(quote!(), |token_stream, asset| {
        asset.attach_token_stream_for_creation(token_stream, name.to_string())
    });
    let mut index = 0;
    asset_creation.append_all(from_world_fields.iter().fold(quote!(), |es, ident| {
        let index_ident = Index::from(index);
        let tokens = quote! (#es #ident : from_world_fields.#index_ident,);
        index += 1;
        tokens
    }));
    let create_function = quote! {
        fn create(world: &mut ::bevy::ecs::world::World) -> Self {
            let from_world_fields = (#prepare_from_world);
            world.resource_scope(
                |world, asset_keys: ::bevy::prelude::Mut<::bevy_asset_loader::dynamic_asset::DynamicAssets>| {
                    #name {
                        #asset_creation
                    }
                },
            )
        }
    };

    let impl_asset_collection = quote! {
        #[automatically_derived]
        #[allow(unused_variables)]
        impl AssetCollection for #name {
            #create_function

            #load_function
        }
    };
    Ok(impl_asset_collection)
}

#[derive(Debug)]
enum ParseFieldError {
    NoAttributes,
    KeyAttributeStandsAlone,
    OnlyDynamicCanBeOptional,
    PathAndPathsAreExclusive,
    WrongAttributeType(proc_macro2::TokenStream, &'static str),
    UnknownAttributeType(proc_macro2::TokenStream),
    UnknownAttribute(proc_macro2::TokenStream),
    MissingAttributes(Vec<String>),
    #[allow(dead_code)]
    Missing2dFeature(proc_macro2::TokenStream),
    #[allow(dead_code)]
    Missing3dFeature(proc_macro2::TokenStream),
    #[allow(dead_code)]
    Missing2dOr3dFeature(proc_macro2::TokenStream),
}

fn parse_field(field: &Field) -> Result<AssetField, Vec<ParseFieldError>> {
    let mut builder = AssetBuilder::default();
    let mut errors = vec![];
    for attr in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident(ASSET_ATTRIBUTE))
    {
        let asset_meta_list = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);

        builder.field_ident = Some(field.clone().ident.unwrap());

        for attribute in asset_meta_list.unwrap() {
            match attribute {
                Meta::List(meta_list)
                    if meta_list
                        .path
                        .is_ident(TextureAtlasAttribute::ATTRIBUTE_NAME)
                        || meta_list
                            .path
                            .is_ident(TextureAtlasAttribute::ATTRIBUTE_NAME_DEPRECATED) =>
                {
                    #[cfg(not(feature = "2d"))]
                    errors.push(ParseFieldError::Missing2dFeature(
                        meta_list.into_token_stream(),
                    ));
                    #[cfg(feature = "2d")]
                    {
                        let texture_atlas_meta_list = meta_list
                            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
                        for attribute in texture_atlas_meta_list.unwrap() {
                            match attribute {
                                Meta::NameValue(named_value) => {
                                    let path = named_value.path.get_ident().unwrap().clone();
                                    if path == TextureAtlasAttribute::TILE_SIZE_X {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(width),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.tile_size_x =
                                                Some(width.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else if path == TextureAtlasAttribute::TILE_SIZE_Y {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(height),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.tile_size_y =
                                                Some(height.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else if path == TextureAtlasAttribute::COLUMNS {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(columns),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.columns =
                                                Some(columns.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else if path == TextureAtlasAttribute::ROWS {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(rows),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.rows =
                                                Some(rows.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else if path == TextureAtlasAttribute::PADDING_X {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(padding_x),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.padding_x =
                                                Some(padding_x.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else if path == TextureAtlasAttribute::PADDING_Y {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(padding_y),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.padding_y =
                                                Some(padding_y.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else if path == TextureAtlasAttribute::OFFSET_X {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(offset_x),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.offset_x =
                                                Some(offset_x.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else if path == TextureAtlasAttribute::OFFSET_Y {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(offset_y),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.offset_y =
                                                Some(offset_y.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else {
                                        errors.push(ParseFieldError::UnknownAttribute(
                                            named_value.into_token_stream(),
                                        ));
                                    }
                                }
                                _ => {
                                    errors.push(ParseFieldError::UnknownAttributeType(
                                        attribute.into_token_stream(),
                                    ));
                                }
                            }
                        }
                    }
                }
                Meta::List(meta_list) if meta_list.path.is_ident(COLLECTION_ATTRIBUTE) => {
                    let collection_meta_list =
                        meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);

                    builder.is_collection = true;
                    for attribute in collection_meta_list.unwrap() {
                        match attribute {
                            Meta::Path(meta_path) => {
                                let path = meta_path.get_ident().unwrap().clone();
                                if path == TYPED_ATTRIBUTE {
                                    builder.is_typed = true;
                                } else if path == MAPPED_ATTRIBUTE {
                                    builder.is_mapped = true;
                                } else {
                                    errors.push(ParseFieldError::UnknownAttribute(
                                        meta_path.into_token_stream(),
                                    ));
                                }
                            }
                            _ => {
                                errors.push(ParseFieldError::UnknownAttributeType(
                                    attribute.into_token_stream(),
                                ));
                            }
                        }
                    }
                }
                Meta::List(meta_list) if meta_list.path.is_ident(PATHS_ATTRIBUTE) => {
                    let paths_meta_list = meta_list
                        .parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated);

                    let mut paths = vec![];
                    for path in paths_meta_list.unwrap() {
                        paths.push(path.value());
                    }
                    builder.asset_paths = Some(paths);
                }
                Meta::List(meta_list)
                    if meta_list.path.is_ident(ImageAttribute::ATTRIBUTE_NAME) =>
                {
                    #[cfg(all(not(feature = "2d"), not(feature = "3d")))]
                    errors.push(ParseFieldError::Missing2dOr3dFeature(
                        meta_list.into_token_stream(),
                    ));
                    #[cfg(any(feature = "2d", feature = "3d"))]
                    {
                        let image_meta_list = meta_list
                            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
                        for attribute in image_meta_list.unwrap() {
                            match attribute {
                                Meta::List(meta_list) => {
                                    let path = meta_list.path.get_ident().unwrap().clone();
                                    if path == ImageAttribute::SAMPLER {
                                        let sampler_meta_list = meta_list
                                            .parse_args_with(
                                                Punctuated::<Meta, Token![,]>::parse_terminated,
                                            )
                                            .unwrap();
                                        for attribute in &sampler_meta_list {
                                            match attribute {
                                                Meta::NameValue(named_value) => {
                                                    let path = named_value
                                                        .path
                                                        .get_ident()
                                                        .unwrap()
                                                        .clone();
                                                    if path == SamplerAttribute::FILTER {
                                                        if let Expr::Path(ExprPath {
                                                            path, ..
                                                        }) = &named_value.value
                                                        {
                                                            let filter_result =
                                                                FilterType::try_from(
                                                                    path.get_ident()
                                                                        .unwrap()
                                                                        .to_string(),
                                                                );

                                                            if let Ok(filter) = filter_result {
                                                                builder.filter = Some(filter);
                                                            } else {
                                                                errors.push(ParseFieldError::UnknownAttribute(
                                                                    named_value.value.clone().into_token_stream(),
                                                                ));
                                                            }
                                                        } else {
                                                            errors.push(
                                                                ParseFieldError::WrongAttributeType(
                                                                    named_value.into_token_stream(),
                                                                    "path",
                                                                ),
                                                            );
                                                        }
                                                    }
                                                }
                                                Meta::Path(path) => {
                                                    let path = path.get_ident().unwrap().clone();
                                                    if path == SamplerAttribute::CLAMP {
                                                        builder.wrap = Some(WrapMode::Clamp);
                                                    } else if path == SamplerAttribute::REPEAT {
                                                        builder.wrap = Some(WrapMode::Repeat);
                                                    } else {
                                                        errors.push(
                                                            ParseFieldError::UnknownAttribute(
                                                                path.into_token_stream(),
                                                            ),
                                                        );
                                                    }
                                                }
                                                Meta::List(_) => {
                                                    errors.push(
                                                        ParseFieldError::WrongAttributeType(
                                                            sampler_meta_list
                                                                .clone()
                                                                .into_token_stream(),
                                                            "name-value or path",
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                Meta::NameValue(named_value) => {
                                    let path = named_value.path.get_ident().unwrap().clone();
                                    if path == ImageAttribute::LAYERS {
                                        if let Expr::Lit(ExprLit {
                                            lit: Lit::Int(layers),
                                            ..
                                        }) = &named_value.value
                                        {
                                            builder.array_texture_layers =
                                                Some(layers.base10_parse::<u32>().unwrap());
                                        } else {
                                            errors.push(ParseFieldError::WrongAttributeType(
                                                named_value.into_token_stream(),
                                                "u32",
                                            ));
                                        }
                                    } else {
                                        errors.push(ParseFieldError::UnknownAttributeType(
                                            path.into_token_stream(),
                                        ));
                                    }
                                }
                                _ => {
                                    errors.push(ParseFieldError::UnknownAttributeType(
                                        attribute.into_token_stream(),
                                    ));
                                }
                            }
                        }
                    }
                }
                Meta::List(meta_list) => errors.push(ParseFieldError::UnknownAttribute(
                    meta_list.into_token_stream(),
                )),
                Meta::NameValue(named_value) if named_value.path.is_ident(PATH_ATTRIBUTE) => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(path),
                        ..
                    }) = &named_value.value
                    {
                        builder.asset_path = Some(path.value());
                    } else {
                        errors.push(ParseFieldError::WrongAttributeType(
                            named_value.into_token_stream(),
                            "str",
                        ));
                    }
                }
                Meta::NameValue(named_value) if named_value.path.is_ident(KEY_ATTRIBUTE) => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(key), ..
                    }) = &named_value.value
                    {
                        builder.key = Some(key.value());
                    } else {
                        errors.push(ParseFieldError::WrongAttributeType(
                            named_value.into_token_stream(),
                            "str",
                        ));
                    }
                }
                Meta::NameValue(named_value) => errors.push(ParseFieldError::UnknownAttribute(
                    named_value.into_token_stream(),
                )),
                Meta::Path(meta_path) if meta_path.is_ident(STANDARD_MATERIAL_ATTRIBUTE) => {
                    #[cfg(not(feature = "3d"))]
                    errors.push(ParseFieldError::Missing3dFeature(
                        meta_path.into_token_stream(),
                    ));
                    #[cfg(feature = "3d")]
                    {
                        builder.is_standard_material = true;
                    }
                }
                Meta::Path(meta_path) if meta_path.is_ident(OPTIONAL_ATTRIBUTE) => {
                    builder.is_optional = true;
                }
                Meta::Path(meta_path) if meta_path.is_ident(COLLECTION_ATTRIBUTE) => {
                    builder.is_collection = true;
                }
                Meta::Path(meta_path) if meta_path.is_ident(TYPED_ATTRIBUTE) => {
                    builder.is_typed = true;
                }
                Meta::Path(meta_path) => errors.push(ParseFieldError::UnknownAttribute(
                    meta_path.into_token_stream(),
                )),
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    builder.build()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
