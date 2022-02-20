use proc_macro2::Ident;

use crate::{ParseFieldError, TextureAtlasAttribute, TEXTURE_ATLAS_ATTRIBUTE};

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
