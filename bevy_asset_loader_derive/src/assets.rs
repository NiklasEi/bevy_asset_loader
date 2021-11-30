use crate::{ParseFieldError, TextureAtlasAttribute, TEXTURE_ATLAS_ATTRIBUTE};
use proc_macro2::Ident;

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

pub(crate) struct BasicAsset {
    pub field_ident: Ident,
    pub asset_path: String,
}

pub(crate) enum Asset {
    Basic(BasicAsset),
    ColorMaterial(BasicAsset),
    Folder(BasicAsset),
    TextureAtlas(TextureAtlasAsset),
}

#[derive(Default)]
pub(crate) struct AssetBuilder {
    pub field_ident: Option<Ident>,
    pub asset_path: Option<String>,
    pub folder_path: Option<String>,
    pub tile_size_x: Option<f32>,
    pub tile_size_y: Option<f32>,
    pub columns: Option<usize>,
    pub rows: Option<usize>,
    pub padding_x: f32,
    pub padding_y: f32,
    pub is_color_material: bool,
    pub is_folder: bool,
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
        if self.asset_path.is_none() && self.folder_path.is_none() {
            return Err(vec![ParseFieldError::NoAttributes]);
        }
        if self.folder_path.is_some() && self.asset_path.is_some() {
            return Err(vec![ParseFieldError::EitherSingleAssetOrFolder]);
        }
        if missing_fields.len() == 4 {
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
            if self.is_color_material {
                return Ok(Asset::ColorMaterial(asset));
            }
            if self.is_folder {
                return Ok(Asset::Folder(asset));
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
                padding_x: self.padding_x,
                padding_y: self.padding_y,
            }));
        }
        Err(vec![ParseFieldError::MissingAttributes(missing_fields)])
    }
}
