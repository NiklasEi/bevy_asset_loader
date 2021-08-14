//! This crate adds support for auto deriving ``bevy_asset_loader::AssetCollection``
//!
//! You do not have to use it directly. Just import ``AssetCollection`` from ``bevy_asset_loader``
//! and use ``#[derive(AssetCollection)]`` to derive the trait.

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens, TokenStreamExt};
use std::option::Option::Some;
use std::result::Result::{Err, Ok};
use syn::{Data, Field, Fields, Lit, Meta, NestedMeta};

/// Derive macro for AssetCollection
///
/// The helper attribute ``asset`` can be used to define the path to the asset file.
#[proc_macro_derive(AssetCollection, attributes(asset))]
pub fn asset_collection_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_asset_collection(ast)
        .unwrap_or_else(to_compile_errors)
        .into()
}

struct BasicAsset {
    field_ident: Ident,
    asset_path: String,
}

enum Asset {
    Basic(BasicAsset),
    TextureAtlas(TextureAtlasAsset),
}

const ASSET_ATTRIBUTE: &str = "asset";
const PATH_ATTRIBUTE: &str = "path";

const TEXTURE_ATLAS_ATTRIBUTE: &str = "texture_atlas";
const TEXTURE_ATLAS_CELL_WIDTH: &str = "cell_width";
const TEXTURE_ATLAS_CELL_HEIGHT: &str = "cell_height";
const TEXTURE_ATLAS_COLUMNS: &str = "columns";
const TEXTURE_ATLAS_ROWS: &str = "rows";

struct TextureAtlasAsset {
    field_ident: Ident,
    asset_path: String,
    cell_width: f32,
    cell_height: f32,
    columns: usize,
    rows: usize,
}

#[derive(Default)]
struct AssetBuilder {
    field_ident: Option<Ident>,
    asset_path: Option<String>,
    cell_width: Option<f32>,
    cell_height: Option<f32>,
    columns: Option<usize>,
    rows: Option<usize>,
}

impl AssetBuilder {
    fn build(self) -> Result<Asset, ParseFieldError> {
        let mut missing_fields = vec![];
        if self.cell_width.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE, TEXTURE_ATLAS_CELL_WIDTH
            ));
        }
        if self.cell_height.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE, TEXTURE_ATLAS_CELL_HEIGHT
            ));
        }
        if self.columns.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE, TEXTURE_ATLAS_COLUMNS
            ));
        }
        if self.rows.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE, TEXTURE_ATLAS_ROWS
            ));
        }
        if self.field_ident.is_none() || self.asset_path.is_none() {
            return Err(ParseFieldError::NoAttributes);
        }
        if missing_fields.len() == 4 {
            return Ok(Asset::Basic(BasicAsset {
                field_ident: self.field_ident.unwrap(),
                asset_path: self.asset_path.unwrap(),
            }));
        }
        if missing_fields.len() < 1 {
            return Ok(Asset::TextureAtlas(TextureAtlasAsset {
                field_ident: self.field_ident.unwrap(),
                asset_path: self.asset_path.unwrap(),
                cell_width: self.cell_width.unwrap(),
                cell_height: self.cell_height.unwrap(),
                columns: self.columns.unwrap(),
                rows: self.rows.unwrap(),
            }));
        }
        Err(ParseFieldError::MissingAttributes(missing_fields))
    }
}

fn impl_asset_collection(
    ast: syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    let name = &ast.ident;

    let mut default_fields: Vec<Ident> = vec![];
    let mut assets: Vec<Asset> = vec![];
    if let Data::Struct(ref data_struct) = ast.data {
        if let Fields::Named(ref named_fields) = data_struct.fields {
            for field in named_fields.named.iter() {
                let asset = parse_field(field);
                match asset {
                    Ok(asset) => assets.push(asset),
                    Err(ParseFieldError::NoAttributes) => {
                        default_fields.push(field.clone().ident.unwrap())
                    }
                    Err(ParseFieldError::MissingAttributes(missing_attributes)) => {
                        return Err(vec![syn::Error::new_spanned(
                            field.into_token_stream(),
                            format!(
                                "Field is missing asset attributes: {}",
                                missing_attributes.join(", ")
                            ),
                        )]);
                    }
                    Err(ParseFieldError::WrongAttributeType(token_stream, expected)) => {
                        return Err(vec![syn::Error::new_spanned(
                            token_stream,
                            format!("Wrong attribute type. Expected '{}'", expected),
                        )]);
                    }
                    Err(ParseFieldError::UnknownAttributeType(token_stream)) => {
                        return Err(vec![syn::Error::new_spanned(
                            token_stream,
                            "Unknown attribute type",
                        )]);
                    }
                    Err(ParseFieldError::UnknownAttribute(token_stream)) => {
                        return Err(vec![syn::Error::new_spanned(
                            token_stream,
                            "Unknown attribute",
                        )]);
                    }
                }
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

    let mut asset_creation = assets.iter().fold(quote!(), |es, asset| match asset {
        Asset::Basic(basic) => {
            let field_ident = basic.field_ident.clone();
            let asset_path = basic.asset_path.clone();
            quote!(#es#field_ident : asset_server.get_handle(#asset_path),)
        }
        Asset::TextureAtlas(texture_asset) => {
            let field_ident = texture_asset.field_ident.clone();
            let asset_path = texture_asset.asset_path.clone();
            let cell_width = texture_asset.cell_width.clone();
            let cell_height = texture_asset.cell_height.clone();
            let columns = texture_asset.columns.clone();
            let rows = texture_asset.rows.clone();
            quote!(
                #es#field_ident : {
                atlases.add(TextureAtlas::from_grid(
                    asset_server.get_handle(#asset_path),
                    Vec2::new(#cell_width, #cell_height),
                    #columns,
                    #rows,
                ))},
            )
        }
    });
    asset_creation.append_all(default_fields.iter().fold(
        quote!(),
        |es, ident| quote! (#es#ident : Default::default()),
    ));

    let asset_loading = assets.iter().fold(quote!(), |es, asset| match asset {
        Asset::Basic(asset) => {
            let asset_path = asset.asset_path.clone();
            quote!(#es handles.push(asset_server.load_untyped(#asset_path));)
        }
        Asset::TextureAtlas(asset) => {
            let asset_path = asset.asset_path.clone();
            quote!(#es handles.push(asset_server.load_untyped(#asset_path));)
        }
    });

    let impl_asset_collection = quote! {
        #[automatically_derived]
        impl AssetCollection for #name {
            fn create(world: &mut World) -> Self {
                let cell = world.cell();
                let asset_server = cell.get_resource::<AssetServer>().expect("Cannot get AssetServer");
                #[cfg(feature = "render")]
                let mut atlases = cell
                    .get_resource_mut::<Assets<TextureAtlas>>()
                    .expect("Cannot get Assets<TextureAtlas>");
                #name {
                    #asset_creation
                }
            }

            fn load(asset_server: &Res<AssetServer>) -> Vec<HandleUntyped> {
                let mut handles = vec![];
                #asset_loading
                handles
            }
        }
    };
    Ok(impl_asset_collection)
}

enum ParseFieldError {
    NoAttributes,
    WrongAttributeType(proc_macro2::TokenStream, &'static str),
    UnknownAttributeType(proc_macro2::TokenStream),
    UnknownAttribute(proc_macro2::TokenStream),
    MissingAttributes(Vec<String>),
}

fn parse_field(field: &Field) -> Result<Asset, ParseFieldError> {
    let mut builder = AssetBuilder::default();
    for attr in field.attrs.iter() {
        if let syn::Meta::List(ref asset_meta_list) = attr.parse_meta().unwrap() {
            if *asset_meta_list.path.get_ident().unwrap() != ASSET_ATTRIBUTE {
                continue;
            }

            for attribute in asset_meta_list.nested.iter() {
                if let NestedMeta::Meta(Meta::NameValue(ref named_value)) = attribute {
                    let path = named_value.path.get_ident().unwrap().clone();

                    if path == PATH_ATTRIBUTE {
                        if let Lit::Str(path_literal) = &named_value.lit {
                            builder.asset_path = Some(path_literal.value());
                            builder.field_ident = Some(field.clone().ident.unwrap());
                        } else {
                            return Err(ParseFieldError::WrongAttributeType(named_value.clone().into_token_stream(), "str"));
                        }
                    }  else {
                        return Err(ParseFieldError::UnknownAttribute(named_value.clone().into_token_stream()))
                    }
                } else if let NestedMeta::Meta(Meta::List(ref meta_list)) = attribute {
                    let path = meta_list.path.get_ident().unwrap().clone();
                    if path == TEXTURE_ATLAS_ATTRIBUTE {
                        for attribute in meta_list.nested.iter() {
                            if let NestedMeta::Meta(Meta::NameValue(ref named_value)) = attribute {
                                let path = named_value.path.get_ident().unwrap().clone();
                                if path == TEXTURE_ATLAS_CELL_WIDTH {
                                    if let Lit::Float(width) = &named_value.lit {
                                        builder.cell_width =
                                            Some(width.base10_parse::<f32>().unwrap())
                                    } else {
                                        return Err(ParseFieldError::WrongAttributeType(named_value.clone().into_token_stream(), "float"));
                                    }
                                } else if path == TEXTURE_ATLAS_CELL_HEIGHT {
                                    if let Lit::Float(height) = &named_value.lit {
                                        builder.cell_height =
                                            Some(height.base10_parse::<f32>().unwrap())
                                    } else {
                                        return Err(ParseFieldError::WrongAttributeType(named_value.clone().into_token_stream(), "float"));
                                    }
                                } else if path == TEXTURE_ATLAS_COLUMNS {
                                    if let Lit::Int(columns) = &named_value.lit {
                                        builder.columns =
                                            Some(columns.base10_parse::<usize>().unwrap())
                                    } else {
                                        return Err(ParseFieldError::WrongAttributeType(named_value.clone().into_token_stream(), "integer"));
                                    }
                                } else if path == TEXTURE_ATLAS_ROWS {
                                    if let Lit::Int(rows) = &named_value.lit {
                                        builder.rows = Some(rows.base10_parse::<usize>().unwrap())
                                    } else {
                                        return Err(ParseFieldError::WrongAttributeType(named_value.clone().into_token_stream(), "integer"));
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(ParseFieldError::UnknownAttribute(meta_list.clone().into_token_stream()))
                    }
                } else {
                    return Err(ParseFieldError::UnknownAttributeType(attribute.clone().into_token_stream()));
                }
            }
        }
    }
    builder.build()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
