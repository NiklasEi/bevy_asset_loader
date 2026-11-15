//! Loading folders with a fixed asset type.

use std::any::TypeId;
use std::fmt;
use std::path::Path;

use bevy_asset::io::ErasedAssetReader;
use bevy_asset::{AssetPath, AssetServer, AssetServerMode, Handle, LoadedFolder};
use bevy_ecs::{resource::Resource, world::World};
use bevy_platform::collections::HashMap;
use futures_lite::StreamExt;

/// Typed folder handles by folder path and asset type.
#[derive(Resource, Default)]
struct TypedFolders(HashMap<(AssetPath<'static>, TypeId), Handle<LoadedFolder>>);

/// Start loading all files in `path` (recursively) as assets of type `asset_type`.
///
/// Files are skipped if no registered loader produces `asset_type` for their extension.
pub fn load_folder_as<'a>(
    world: &mut World,
    path: impl Into<AssetPath<'a>>,
    asset_type: TypeId,
) -> Handle<LoadedFolder> {
    let path = path.into().into_owned();
    let asset_server = world.resource::<AssetServer>().clone();
    let key = (path.clone(), asset_type);
    if let Some(handle) = world
        .get_resource::<TypedFolders>()
        .and_then(|folders| folders.0.get(&key))
        && !asset_server.load_state(handle).is_failed()
    {
        return handle.clone();
    }

    let handle = asset_server.add_async(load_folder(asset_server.clone(), path, asset_type));
    world
        .get_resource_or_init::<TypedFolders>()
        .0
        .insert(key, handle.clone());
    handle
}

/// The handle of a folder previously loaded with [`load_folder_as`].
pub fn get_folder_as<'a>(
    world: &World,
    path: impl Into<AssetPath<'a>>,
    asset_type: TypeId,
) -> Option<Handle<LoadedFolder>> {
    let key = (path.into().into_owned(), asset_type);
    world.get_resource::<TypedFolders>()?.0.get(&key).cloned()
}

async fn load_folder(
    server: AssetServer,
    path: AssetPath<'static>,
    asset_type: TypeId,
) -> Result<LoadedFolder, TypedFolderError> {
    let error = |message: String| TypedFolderError {
        path: path.clone(),
        message,
    };
    let source = server
        .get_source(path.source())
        .map_err(|err| error(err.to_string()))?;
    let reader = match server.mode() {
        AssetServerMode::Unprocessed => source.reader(),
        AssetServerMode::Processed => source
            .processed_reader()
            .map_err(|err| error(err.to_string()))?,
    };

    let mut handles = Vec::new();
    let mut directories = vec![path.path().to_path_buf()];
    while let Some(directory) = directories.pop() {
        let is_directory = reader
            .is_directory(&directory)
            .await
            .map_err(|err| error(err.to_string()))?;
        if !is_directory {
            return Err(error(format!(
                "'{}' is not a directory",
                directory.display()
            )));
        }
        let mut entries = reader
            .read_directory(&directory)
            .await
            .map_err(|err| error(err.to_string()))?;
        while let Some(entry) = entries.next().await {
            if is_directory_entry(reader, &entry).await? {
                directories.push(entry);
                continue;
            }
            let Some(entry_path) = entry.to_str() else {
                continue;
            };
            let asset_path = AssetPath::parse(entry_path)
                .into_owned()
                .with_source(source.id());
            if loads_as(&server, &asset_path, asset_type).await {
                handles.push(server.load_builder().load_erased(asset_type, asset_path));
            }
        }
    }

    Ok(LoadedFolder { handles })
}

async fn is_directory_entry(
    reader: &dyn ErasedAssetReader,
    path: &Path,
) -> Result<bool, TypedFolderError> {
    reader
        .is_directory(path)
        .await
        .map_err(|err| TypedFolderError {
            path: AssetPath::from_path_buf(path.to_path_buf()),
            message: err.to_string(),
        })
}

/// Whether a registered loader for the file's extension produces `asset_type`.
async fn loads_as(server: &AssetServer, path: &AssetPath<'_>, asset_type: TypeId) -> bool {
    if let Ok(loader) = server.get_path_asset_loader(path.clone()).await
        && loader.asset_type_id() == asset_type
    {
        return true;
    }
    let Ok(loader) = server.get_asset_loader_with_asset_type_id(asset_type).await else {
        return false;
    };
    let Some(extension) = path.get_full_extension() else {
        return false;
    };
    // "tar.gz" also matches a loader for "gz"
    std::iter::successors(Some(extension), |extension| {
        extension.split_once('.').map(|(_, rest)| rest)
    })
    .any(|extension| loader.extensions().contains(&extension))
}

/// Error while loading a typed folder.
#[derive(Debug)]
pub struct TypedFolderError {
    path: AssetPath<'static>,
    message: String,
}

impl fmt::Display for TypedFolderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to load folder '{}': {}", self.path, self.message)
    }
}

impl std::error::Error for TypedFolderError {}
