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
pub(crate) struct DynamicAssetField {
    pub field_ident: Ident,
    pub key: String,
}

#[derive(PartialEq, Debug)]
pub(crate) enum AssetField {
    Basic(BasicAssetField),
    Dynamic(DynamicAssetField),
    OptionalDynamic(DynamicAssetField),
    DynamicFolder(DynamicAssetField),
    StandardMaterial(BasicAssetField),
    Folder(BasicAssetField),
    TextureAtlas(TextureAtlasAssetField),
}

impl AssetField {
    pub(crate) fn attach_token_stream_for_creation(
        &self,
        token_stream: TokenStream,
    ) -> TokenStream {
        #[allow(unused_mut, unused_assignments)]
        let mut conditional_dynamic_asset_collections = quote! {};

        #[cfg(feature = "render")]
        {
            conditional_dynamic_asset_collections = quote! {
            bevy_asset_loader::DynamicAsset::TextureAtlas {
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
            bevy_asset_loader::DynamicAsset::StandardMaterial { path } => materials.add(asset_server.get_handle::<bevy::prelude::Image, &String>(path).into()).clone_untyped(),};
        }

        match self {
            AssetField::Basic(basic) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                quote!(#token_stream #field_ident : asset_server.get_handle(#asset_path),)
            }
            AssetField::Folder(basic) => {
                let field_ident = basic.field_ident.clone();
                let asset_path = basic.asset_path.clone();
                quote!(#token_stream #field_ident : asset_server.load_folder(#asset_path).unwrap(),)
            }
            AssetField::Dynamic(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                quote!(#token_stream #field_ident : {
                let asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                let handle = match asset {
                    bevy_asset_loader::DynamicAsset::File { path } => asset_server.get_handle_untyped(path),
                    #conditional_dynamic_asset_collections
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
                        bevy_asset_loader::DynamicAsset::File { path } => asset_server.get_handle_untyped(path),
                        #conditional_dynamic_asset_collections
                    };
                    handle.typed()
                })
            },)
            }
            AssetField::DynamicFolder(dynamic) => {
                let field_ident = dynamic.field_ident.clone();
                let asset_key = dynamic.key.clone();
                quote!(#token_stream #field_ident : {
                let asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                match asset {
                    bevy_asset_loader::DynamicAsset::File { path } => asset_server.load_folder(path).unwrap(),
                    _ => panic!("The asset '{}' cannot be loaded as a folder, because it is not of the type 'File'", #asset_key)
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
        }
    }

    pub(crate) fn attach_token_stream_for_loading(&self, token_stream: TokenStream) -> TokenStream {
        match self {
            AssetField::Basic(asset) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream handles.push(asset_server.load_untyped(#asset_path));)
            }
            AssetField::Folder(asset) => {
                let asset_path = asset.asset_path.clone();
                quote!(#token_stream asset_server.load_folder(#asset_path).unwrap().drain(..).for_each(|handle| handles.push(handle));)
            }
            AssetField::Dynamic(dynamic) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream handles.push({
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                        asset_server.load_untyped(dynamic_asset.get_file_path())
                    });
                )
            }
            AssetField::OptionalDynamic(dynamic) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream {
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into());
                        if let Some(dynamic_asset) = dynamic_asset {
                            handles.push(asset_server.load_untyped(dynamic_asset.get_file_path()));
                        }
                    }
                )
            }
            AssetField::DynamicFolder(dynamic) => {
                let asset_key = dynamic.key.clone();
                quote!(
                    #token_stream {
                        let dynamic_asset = asset_keys.get_asset(#asset_key.into()).unwrap_or_else(|| panic!("Failed to get asset for key '{}'", #asset_key));
                        asset_server.load_folder(dynamic_asset.get_file_path()).unwrap().drain(..).for_each(|handle| handles.push(handle));
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
        }
    }
}

#[derive(Default)]
pub(crate) struct AssetBuilder {
    pub field_ident: Option<Ident>,
    pub asset_path: Option<String>,
    pub is_standard_material: bool,
    pub is_optional: bool,
    pub is_folder: bool,
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
        if self.asset_path.is_none() && self.key.is_none() {
            return Err(vec![ParseFieldError::NoAttributes]);
        }
        if self.key.is_some()
            && (self.asset_path.is_some()
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
        if missing_fields.len() == 4 {
            if self.key.is_some() {
                return if self.is_optional {
                    // Todo support optional folder?
                    Ok(AssetField::OptionalDynamic(DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key: self.key.unwrap(),
                    }))
                } else if self.is_folder {
                    Ok(AssetField::DynamicFolder(DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key: self.key.unwrap(),
                    }))
                } else {
                    Ok(AssetField::Dynamic(DynamicAssetField {
                        field_ident: self.field_ident.unwrap(),
                        key: self.key.unwrap(),
                    }))
                };
            }
            if self.is_folder {
                return Ok(AssetField::Folder(BasicAssetField {
                    field_ident: self.field_ident.unwrap(),
                    asset_path: self.asset_path.unwrap(),
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
            AssetField::Folder(BasicAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/folder".to_owned()
            })
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
            AssetField::DynamicFolder(DynamicAssetField {
                field_ident: Ident::new("test", Span::call_site()),
                key: "some.asset.key".to_owned(),
            }),
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
}
