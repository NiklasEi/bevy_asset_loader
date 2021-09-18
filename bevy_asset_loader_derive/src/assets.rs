use proc_macro2::Ident;

pub struct TextureAtlasAsset {
    pub field_ident: Ident,
    pub asset_path: String,
    pub tile_size_x: f32,
    pub tile_size_y: f32,
    pub columns: usize,
    pub rows: usize,
    pub padding_x: f32,
    pub padding_y: f32,
}

pub struct BasicAsset {
    pub field_ident: Ident,
    pub asset_path: String,
}

pub enum Asset {
    Basic(BasicAsset),
    ColorMaterial(BasicAsset),
    TextureAtlas(TextureAtlasAsset),
}
