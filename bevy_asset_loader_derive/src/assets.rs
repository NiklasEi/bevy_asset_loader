use crate::{ParseFieldError, TextureAtlasAttribute, TEXTURE_ATLAS_ATTRIBUTE};
use proc_macro2::Ident;

#[derive(PartialEq, Debug)]
pub(crate) struct TextureAtlasAsset {
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
pub(crate) struct BasicAsset {
    pub field_ident: Ident,
    pub asset_path: String,
}

#[derive(PartialEq, Debug)]
pub(crate) struct DynamicAsset {
    pub field_ident: Ident,
    pub key: String,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Asset {
    Basic(BasicAsset),
    Dynamic(DynamicAsset),
    StandardMaterial(BasicAsset),
    Folder(BasicAsset),
    TextureAtlas(TextureAtlasAsset),
}

#[derive(Default)]
pub(crate) struct AssetBuilder {
    pub field_ident: Option<Ident>,
    pub asset_path: Option<String>,
    pub folder_path: Option<String>,
    pub is_standard_material: bool,
    pub key: Option<String>,
    pub tile_size_x: Option<f32>,
    pub tile_size_y: Option<f32>,
    pub columns: Option<usize>,
    pub rows: Option<usize>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
}

impl AssetBuilder {
    pub(crate) fn build(self) -> Result<Asset, Vec<ParseFieldError>> {
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
        if self.asset_path.is_none() && self.folder_path.is_none() && self.key.is_none() {
            return Err(vec![ParseFieldError::NoAttributes]);
        }
        if self.key.is_some()
            && (self.folder_path.is_some()
                || self.asset_path.is_some()
                || missing_fields.len() < 4
                || self.padding_x.is_some()
                || self.padding_y.is_some()
                || self.is_standard_material)
        {
            return Err(vec![ParseFieldError::KeyAttributeStandsAlone]);
        }
        if self.folder_path.is_some() && self.asset_path.is_some() {
            return Err(vec![ParseFieldError::EitherSingleAssetOrFolder]);
        }
        if missing_fields.len() == 4 {
            if self.key.is_some() {
                return Ok(Asset::Dynamic(DynamicAsset {
                    field_ident: self.field_ident.unwrap(),
                    key: self.key.unwrap(),
                }));
            }
            if self.folder_path.is_some() {
                return Ok(Asset::Folder(BasicAsset {
                    field_ident: self.field_ident.unwrap(),
                    asset_path: self.folder_path.unwrap(),
                }));
            }
            let asset = BasicAsset {
                field_ident: self.field_ident.unwrap(),
                asset_path: self.asset_path.unwrap(),
            };
            if self.is_standard_material {
                return Ok(Asset::StandardMaterial(asset));
            }
            return Ok(Asset::Basic(asset));
        }
        if missing_fields.is_empty() {
            return Ok(Asset::TextureAtlas(TextureAtlasAsset {
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
        let mut builder = AssetBuilder::default();
        builder.field_ident = Some(Ident::new("test", Span::call_site()));
        builder.asset_path = Some("some/image.png".to_owned());

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            Asset::Basic(BasicAsset {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned()
            })
        );
    }

    #[test]
    fn standard_material() {
        let mut builder = AssetBuilder::default();
        builder.field_ident = Some(Ident::new("test", Span::call_site()));
        builder.asset_path = Some("some/image.png".to_owned());
        builder.is_standard_material = true;

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            Asset::StandardMaterial(BasicAsset {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/image.png".to_owned()
            })
        );
    }

    #[test]
    fn folder() {
        let mut builder = AssetBuilder::default();
        builder.field_ident = Some(Ident::new("test", Span::call_site()));
        builder.folder_path = Some("some/folder".to_owned());

        let asset = builder.build().expect("This should be a valid BasicAsset");
        assert_eq!(
            asset,
            Asset::Folder(BasicAsset {
                field_ident: Ident::new("test", Span::call_site()),
                asset_path: "some/folder".to_owned()
            })
        );
    }

    #[test]
    fn dynamic_asset() {
        let mut builder = AssetBuilder::default();
        builder.field_ident = Some(Ident::new("test", Span::call_site()));
        builder.key = Some("some.asset.key".to_owned());

        let asset = builder
            .build()
            .expect("This should be a valid DynamicAsset");
        assert_eq!(
            asset,
            Asset::Dynamic(DynamicAsset {
                field_ident: Ident::new("test", Span::call_site()),
                key: "some.asset.key".to_owned()
            })
        );
    }

    #[test]
    fn texture_atlas() {
        let mut builder = AssetBuilder::default();
        builder.field_ident = Some(Ident::new("test", Span::call_site()));
        builder.asset_path = Some("some/folder".to_owned());
        builder.tile_size_x = Some(100.);
        builder.tile_size_y = Some(50.);
        builder.columns = Some(10);
        builder.rows = Some(5);
        builder.padding_x = Some(2.);

        let asset = builder
            .build()
            .expect("This should be a valid TextureAtlasAsset");
        assert_eq!(
            asset,
            Asset::TextureAtlas(TextureAtlasAsset {
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
    fn dynamic_asset_does_not_accept_more_attributes() {
        let mut builder = asset_builder_dynamic();
        builder.asset_path = Some("path".to_owned());
        assert!(builder.build().is_err());

        let mut builder = asset_builder_dynamic();
        builder.folder_path = Some("path".to_owned());
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
    }

    fn asset_builder_dynamic() -> AssetBuilder {
        let mut builder = AssetBuilder::default();
        builder.field_ident = Some(Ident::new("test", Span::call_site()));
        builder.key = Some("some.asset.key".to_owned());

        builder
    }
}
