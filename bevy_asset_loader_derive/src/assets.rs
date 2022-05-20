use crate::{ParseFieldError, TextureAtlasAttribute, TEXTURE_ATLAS_ATTRIBUTE};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

#[derive(PartialEq, Debug)]
pub(crate) struct TextureAtlasAssetField {
    pub field_ident: Ident,
    pub asset_path: String,
    pub tile_size_x: f32,
    pub tile_size_y: f32,
    pub columns: usize,
    pub rows: usize,
    pub padding_x: f32,
    pub padding_y: f32,
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
    Folder(BasicAssetField, Typed),
    Files(MultipleFilesField, Typed),
    TextureAtlas(TextureAtlasAssetField),
    StandardMaterial(BasicAssetField),
    Dynamic(DynamicAssetField),
    OptionalDynamic(DynamicAssetField),
    DynamicFolder(DynamicAssetField, Typed),
    DynamicFiles(DynamicAssetField, Typed),
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

impl AssetField {
    pub(crate) fn attach_token_stream_for_creation(
        &self,
        token_stream: TokenStream,
    ) -> TokenStream {
        #[allow(unused_mut, unused_assignments)]
        let mut conditional_dynamic_asset_collections = quote! {};

        #[cfg(feature = "2d")]
        {
            let conditional_2d = quote! {
            ::bevy_asset_loader::DynamicAsset::TextureAtlas {
                path,
                tile_size_x,
                tile_size_y,
                columns,
                rows,
                padding_x,
                padding_y,
            } => atlases.add(TextureAtlas::from_grid_with_padding(
                asset_server.get_handle(path),
                Vec2::new(*tile_size_x, *tile_size_y),
                *columns,
                *rows,
                Vec2::new(padding_x.unwrap_or(0.), padding_y.unwrap_or(0.)),
            )).clone_untyped(),
            };
            conditional_dynamic_asset_collections.extend(conditional_2d);
        }
        #[cfg(feature = "3d")]
        {
            let conditional_3d = quote! {
            ::bevy_asset_loader::DynamicAsset::StandardMaterial { path } =>
                materials.add(asset_server.get_handle::<bevy::prelude::Image, &String>(path).into()).clone_untyped(),
            };
            conditional_dynamic_asset_collections.extend(conditional_3d);
        }

        match self {
            AssetField::Basic(basic) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                quote!(#token_stream #field_ident : asset_server.get_handle(#asset_path),)
            }
            AssetField::Folder(basic, typed) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                match typed {
                    Typed::Yes => {
                        quote!(#token_stream #field_ident : asset_server.load_folder(#asset_path)
                            .unwrap()
                            .drain(..)
                            .map(|handle| handle.typed())
                            .collect(),
                        )
                    }
                    Typed::No => {
                        quote!(#token_stream #field_ident : asset_server.load_folder(#asset_path).unwrap(),)
                    }
                }
            }
            AssetField::Dynamic(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                    let handle = match asset {
                        ::bevy_asset_loader::DynamicAsset::File { path } => asset_server.get_handle_untyped(path),
                        #conditional_dynamic_asset_collections
                        _ => panic!("The dynamic asset '{}' cannot be created (expected `File`, `StandardMaterial`, or `TextureAtlas`)", #asset_key)
                    };
                    handle.typed()
                },)
            }
            AssetField::OptionalDynamic(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into());
                    asset.map(|asset| {
                        let handle = match asset {
                            ::bevy_asset_loader::DynamicAsset::File { path } => asset_server.get_handle_untyped(path),
                            #conditional_dynamic_asset_collections
                            _ => panic!("The dynamic asset '{}' cannot be created (expected `File`, `StandardMaterial`, or `TextureAtlas`)", #asset_key)
                        };
                        handle.typed()
                    })
                },)
            }
            AssetField::DynamicFolder(dynamic, typed) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                let load = match typed {
                    Typed::Yes => {
                        quote!(::bevy_asset_loader::DynamicAsset::Folder { path } => asset_server.load_folder(path)
                            .unwrap()
                            .drain(..)
                            .map(|handle| handle.typed())
                            .collect()
                        )
                    }
                    Typed::No => {
                        quote!(::bevy_asset_loader::DynamicAsset::Folder { path } => asset_server.load_folder(path).unwrap())
                    }
                };
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                    match asset {
                        #load,
                        _ => panic!("The dynamic asset '{}' cannot be created (expected `Folder`)", #asset_key)
                    }
                },)
            }
            AssetField::DynamicFiles(dynamic, typed) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                let load = match typed {
                    Typed::Yes => {
                        quote!(::bevy_asset_loader::DynamicAsset::Files { paths } => paths
                            .iter()
                            .map(|path| asset_server.load_untyped(path))
                            .drain(..)
                            .map(|handle| handle.typed())
                            .collect()
                        )
                    }
                    Typed::No => {
                        quote!(::bevy_asset_loader::DynamicAsset::Files { paths } => paths
                            .iter()
                            .map(|path| asset_server.load(path))
                            .collect()
                        )
                    }
                };
                quote!(#token_stream #field_ident : {
                    let asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                    match asset {
                        #load,
                        _ => panic!("The dynamic asset '{}' cannot be loaded (expected `Files`)", #asset_key)
                    }
                },)
            }
            AssetField::StandardMaterial(basic) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                quote!(#token_stream #field_ident : materials.add(asset_server.get_handle(#asset_path).into()),)
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
                quote!(
                    #token_stream #field_ident : {
                    atlases.add(TextureAtlas::from_grid_with_padding(
                        asset_server.get_handle(#asset_path),
                        Vec2::new(#tile_size_x, #tile_size_y),
                        #columns,
                        #rows,
                        Vec2::new(#padding_x, #padding_y),
                    ))},
                )
            }
            AssetField::Files(files, typed) => {
                let field_ident = files.field_ident.clone();
                let asset_paths = files.asset_paths.clone();
                match typed {
                    Typed::Yes => {
                        quote!(#token_stream #field_ident : vec![#(asset_server.load(#asset_paths)),*],)
                    }
                    Typed::No => {
                        quote!(#token_stream #field_ident : vec![#(asset_server.load_untyped(#asset_paths)),*],)
                    }
                }
            }
        }
    }

    pub(crate) fn attach_token_stream_for_loading(&self, token_stream: TokenStream) -> TokenStream {
        match self {
            AssetField::Basic(asset) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream handles.push(asset_server.load_untyped(#asset_path));)
            }
            AssetField::Folder(asset, _) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream asset_server.load_folder(#asset_path).unwrap().drain(..).for_each(|handle| handles.push(handle));)
            }
            AssetField::OptionalDynamic(dynamic) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream {
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into());
                        if let Some(dynamic_asset) = dynamic_asset {
                            handles.extend(dynamic_asset.load_untyped(&asset_server));
                        }
                    }
                )
            }
            AssetField::Dynamic(dynamic)
            | AssetField::DynamicFolder(dynamic, _)
            | AssetField::DynamicFiles(dynamic, _) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream {
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                        handles.extend(dynamic_asset.load_untyped(&asset_server));
                    }
                )
            }
            AssetField::StandardMaterial(asset) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream handles.push(asset_server.load_untyped(#asset_path));)
            }
            AssetField::TextureAtlas(asset) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream handles.push(asset_server.load_untyped(#asset_path));)
            }
            AssetField::Files(assets, _) => {
                let asset_paths = assets.asset_paths.clone();
                quote!(#token_stream #(handles.push(asset_server.load_untyped(#asset_paths)));*;)
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
    pub is_folder: bool,
    pub is_typed: bool,
    pub key: Option<String>,
    pub tile_size_x: Option<f32>,
    pub tile_size_y: Option<f32>,
    pub columns: Option<usize>,
    pub rows: Option<usize>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
}

impl AssetBuilder {
    pub(crate) fn build(self) -> Result<AssetField, Vec<ParseFieldError>> {
        let mut missing_fields = vec![];
        if self.tile_size_x.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE,
                TextureAtlasAttribute::TILE_SIZE_X
            ));
        }
        if self.tile_size_y.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE,
                TextureAtlasAttribute::TILE_SIZE_Y
            ));
        }
        if self.columns.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE,
                TextureAtlasAttribute::COLUMNS
            ));
        }
        if self.rows.is_none() {
            missing_fields.push(format!(
                "{}/{}",
                TEXTURE_ATLAS_ATTRIBUTE,
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
                    // Todo support optional folder?
                    Ok(AssetField::OptionalDynamic(DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key: self.key.unwrap(),
                    }))
                } else if self.is_folder {
                    Ok(AssetField::DynamicFolder(
                        DynamicAssetField {
                            field_ident: self.field_ident.unwrap(),
                            key: self.key.unwrap(),
                        },
                        self.is_typed.into(),
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
                ));
            }
            if self.is_folder {
                return Ok(AssetField::Folder(
                    BasicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        asset_path: self.asset_path.unwrap(),
                    },
                    self.is_typed.into(),
                ));
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
                tile_size_x: self.tile_size_x.unwrap(),
                tile_size_y: self.tile_size_y.unwrap(),
                columns: self.columns.unwrap(),
                rows: self.rows.unwrap(),
                padding_x: self.padding_x.unwrap_or_default(),
                padding_y: self.padding_y.unwrap_or_default(),
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
            is_folder: true,
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
                Typed::No
            )
        );

        let builder = AssetBuilder {
            field_ident: Some(Ident::new("test", Span::call_site())),
            asset_path: Some("some/folder".to_owned()),
            is_folder: true,
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
                Typed::Yes
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
                Typed::No
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
                Typed::Yes
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
                tile_size_x: 100.0,
                tile_size_y: 50.0,
                columns: 10,
                rows: 5,
                padding_x: 2.0,
                padding_y: 0.0
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
        builder.is_optional = true;
        let asset = builder
            .build()
            .expect("This should be a valid TextureAtlasAsset");
        assert_eq!(
            asset,
            AssetField::OptionalDynamic(DynamicAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                key: "some.asset.key".to_owned(),
            }),
            "Dynamic asset with 'optional' attribute should yield 'AssetField::OptionalDynamic'"
        );

        let mut builder = asset_builder_dynamic();
        builder.is_folder = true;
        let asset = builder
            .build()
            .expect("This should be a valid TextureAtlasAsset");
        assert_eq!(
            asset,
            AssetField::DynamicFolder(
                DynamicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    key: "some.asset.key".to_owned(),
                },
                Typed::No
            ),
            "Dynamic asset with 'folder' attribute should yield 'AssetField::DynamicFolder'"
        );

        let mut builder = asset_builder_dynamic();
        builder.is_folder = true;
        builder.is_typed = true;
        let asset = builder
            .build()
            .expect("This should be a valid TextureAtlasAsset");
        assert_eq!(
            asset,
            AssetField::DynamicFolder(
                DynamicAssetField {
                    field_ident: Ident::new("test", Span::call_site()),
                    key: "some.asset.key".to_owned(),
                },
                Typed::Yes
            ),
            "Dynamic asset with 'folder' attribute should yield 'AssetField::DynamicFolder'"
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
