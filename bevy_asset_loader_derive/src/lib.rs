extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn;
use syn::{Data, Fields, Lit, NestedMeta, Meta};

#[proc_macro_derive(AssetCollection, attributes(asset))]
pub fn asset_collection_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_asset_collection(ast).unwrap_or_else(to_compile_errors).into()
}

struct Asset {
    field_ident: Ident,
    asset_path: String
}

fn impl_asset_collection(ast: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    let name = &ast.ident;

    let mut assets: Vec<Asset> = vec![];
    match ast.data {
        Data::Struct(ref data_struct) => {
            match data_struct.fields {
                Fields::Named(ref named_fields) => {
                    'fields: for field in named_fields.named.iter() {
                        'attributes: for attr in field.attrs.iter() {
                            match attr.parse_meta().unwrap() {
                                syn::Meta::List(ref asset_meta_list) => {
                                    if asset_meta_list.path.get_ident()
                                        .unwrap()
                                        .to_string()
                                        != "asset"
                                    {
                                        continue 'attributes;
                                    }

                                    for attribute in asset_meta_list.nested.iter() {
                                        if let NestedMeta::Meta(Meta::NameValue(ref named_value)) = attribute {
                                            if named_value.path.get_ident()
                                                .unwrap()
                                                .to_string()
                                                != "path"
                                            {
                                                continue;
                                            }
                                            if let Lit::Str(path_literal) = &named_value.lit {
                                                assets.push(Asset { field_ident: field.clone().ident.unwrap(), asset_path: path_literal.value()});
                                                continue 'fields;
                                            }
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                _ => {}
            }
        },
        _ => {}
    }

    let asset_creation = assets
        .iter()
        .fold(quote!(), |es, Asset {field_ident, asset_path}| quote!(#es#field_ident : asset_server.get_handle(#asset_path),));

    let asset_loading = assets
        .iter()
        .fold(quote!(), |es, Asset {field_ident: _, asset_path}| quote!(#es asset_server.load_untyped(#asset_path),));


    let gen = quote! {
        #[automatically_derived]
        impl AssetCollection for #name {
            fn create(asset_server: &Res<AssetServer>) -> Self {
                #name {
                    #asset_creation
                }
            }

            fn load(asset_server: &Res<AssetServer>) -> Vec<HandleUntyped> {
                vec![#asset_loading]
            }
        }
    };
    Ok(gen)
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
