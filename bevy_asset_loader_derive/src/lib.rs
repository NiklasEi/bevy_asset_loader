//! This crate adds support for auto deriving ``bevy_asset_loader::AssetCollection``
//!
//! You do not have to use it directly. Just import ``AssetCollection`` from ``bevy_asset_loader``
//! and use ``#[derive(AssetCollection)]`` to derive the trait.

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Data, Fields, Lit, Meta, NestedMeta};

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

struct Asset {
    field_ident: Ident,
    asset_path: String,
}

fn impl_asset_collection(
    ast: syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    let name = &ast.ident;

    let mut fields = 0;
    let mut assets: Vec<Asset> = vec![];
    if let Data::Struct(ref data_struct) = ast.data {
        if let Fields::Named(ref named_fields) = data_struct.fields {
            'fields: for field in named_fields.named.iter() {
                fields += 1;
                'attributes: for attr in field.attrs.iter() {
                    if let syn::Meta::List(ref asset_meta_list) = attr.parse_meta().unwrap() {
                        if *asset_meta_list.path.get_ident().unwrap() != "asset" {
                            continue 'attributes;
                        }

                        for attribute in asset_meta_list.nested.iter() {
                            if let NestedMeta::Meta(Meta::NameValue(ref named_value)) = attribute {
                                if *named_value.path.get_ident().unwrap() != "path" {
                                    continue;
                                }
                                if let Lit::Str(path_literal) = &named_value.lit {
                                    assets.push(Asset {
                                        field_ident: field.clone().ident.unwrap(),
                                        asset_path: path_literal.value(),
                                    });
                                    continue 'fields;
                                }
                            }
                        }
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

    if assets.len() != fields {
        return Err(vec![syn::Error::new_spanned(&ast.into_token_stream(), "To auto derive AssetCollection every field should have an asset attribute containing a path")]);
    }

    let asset_creation = assets.iter().fold(
        quote!(),
        |es,
         Asset {
             field_ident,
             asset_path,
         }| quote!(#es#field_ident : asset_server.get_handle(#asset_path),),
    );

    let asset_loading = assets.iter().fold(
        quote!(),
        |es,
         Asset {
             field_ident: _,
             asset_path,
         }| quote!(#es handles.push(asset_server.load_untyped(#asset_path));),
    );

    let gen = quote! {
        #[automatically_derived]
        impl AssetCollection for #name {
            fn create(asset_server: &Res<AssetServer>) -> Self {
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
    Ok(gen)
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
