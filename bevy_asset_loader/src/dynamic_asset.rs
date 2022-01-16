/// These asset variants can be loaded from configuration files. They will then replace
/// a dynamic asset based on their keys.
pub enum DynamicAsset {
    /// A dynamic asset directly loaded from a single file
    File {
        /// Asset file path
        path: String,
    },
    /// A dynamic standard material asset directly loaded from an image file
    #[cfg(feature = "render")]
    StandardMaterial {
        /// Asset file path
        path: String,
    },
    /// A dynamic texture atlas asset loaded form a sprite sheet
    #[cfg(feature = "render")]
    TextureAtlas {
        /// Asset file path
        path: String,
        /// The image width in pixels
        tile_size_x: f32,
        /// The image height in pixels
        tile_size_y: f32,
        /// Columns on the sprite sheet
        columns: usize,
        /// Rows on the sprite sheet
        rows: usize,
        /// Padding between columns in pixels
        padding_x: f32,
        /// Padding between rows in pixels
        padding_y: f32,
    },
}

impl DynamicAsset {
    /// Path to the asset file of the dynamic asset
    pub fn get_file_path(&self) -> &str {
        match self {
            DynamicAsset::File { path } => path,
            #[cfg(feature = "render")]
            DynamicAsset::StandardMaterial { path } => path,
            #[cfg(feature = "render")]
            DynamicAsset::TextureAtlas { path, .. } => path,
        }
    }
}
