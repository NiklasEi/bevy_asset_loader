use std::borrow::Borrow;

use bevy::asset::AssetPath;

/// A type that can be used as key for mapped asset collection.
///
/// # `String` and `Box<str>`
///
/// Both [`String`] and [`Box<str>`] implements [`MapKey`] by using
/// the path of the asset as the key.
pub trait MapKey {
    /// Creates the key from the path of the asset.
    fn from_asset_path(path: &AssetPath) -> Self;
}

/// Implements extra traits and methods for the key types.
macro_rules! impl_map_key_extras {
    ($Key:ty) => {
        impl AsRef<str> for $Key {
            #[inline]
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        // Note: required by `HashMap::get` to being able to use &str.
        impl Borrow<str> for $Key {
            #[inline]
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl From<$Key> for Box<str> {
            #[inline]
            fn from(key: $Key) -> Self {
                key.0
            }
        }

        impl From<$Key> for String {
            #[inline]
            fn from(key: $Key) -> Self {
                key.0.into()
            }
        }
    };
}

/// A [`MapKey`] that uses the [`file_name`] of the asset's path as key.
///
/// [`file_name`]: std::path::Path::file_name
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileName(Box<str>);

impl_map_key_extras!(FileName);

impl MapKey for FileName {
    #[inline]
    fn from_asset_path(path: &AssetPath) -> Self {
        Self(
            path.path()
                .file_name()
                .unwrap()
                .to_str()
                .expect("Path should be valid UTF-8")
                .into(),
        )
    }
}

/// A [`MapKey`] that uses the [`file_stem`] of the asset's path as key.
///
/// [`file_stem`]: std::path::Path::file_stem
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileStem(Box<str>);

impl_map_key_extras!(FileStem);

impl MapKey for FileStem {
    #[inline]
    fn from_asset_path(path: &AssetPath) -> Self {
        Self(
            path.path()
                .file_stem()
                .unwrap()
                .to_str()
                .expect("Path should be valid UTF-8")
                .into(),
        )
    }
}

impl MapKey for String {
    #[inline]
    fn from_asset_path(path: &AssetPath) -> Self {
        path_slash::PathExt::to_slash(path.path())
            .expect("Path should be valid UTF-8")
            .into()
    }
}

impl MapKey for Box<str> {
    #[inline]
    fn from_asset_path(path: &AssetPath) -> Self {
        path_slash::PathExt::to_slash(path.path())
            .expect("Path should be valid UTF-8")
            .into()
    }
}
