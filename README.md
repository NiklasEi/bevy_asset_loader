# Bevy asset loader

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/crates/l/bevy_asset_loader)](https://github.com/NiklasEi/bevy_asset_loader#license)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

This [Bevy][bevy] plugin reduces boilerplate for handling game assets. The crate offers the derivable `AssetCollection` trait and can automatically load structs that implement it. Asset collections contain handles to your game assets and are available to your systems as resources after loading.

In most cases you will want to load your asset collections during loading states (think loading screens). During such a state, all assets are loaded and their loading process is observed. Only when asset collections can be build with fully loaded asset handles, the collections are inserted as resources. If you do not want to use a loading state, asset collections can still result in cleaner code and improved maintainability (see the ["usage without a loading state"](#usage-without-a-loading-state) section).

_The `main` branch and the latest release support Bevy version `0.11` (see [version table](#compatible-bevy-versions))_

## Loading states

A loading state is responsible for managing the loading process during a configurable Bevy state (see [the cheatbook on states][cheatbook-states]).

If your `LoadingState` is set up, you can start your game logic from the next state and use the asset collections as resources in your systems. The loading state guarantees that all handles in your collections are fully loaded by the time the next state starts.

```rust ignore
app
    .add_state::<GameState>()
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Next)
    )
    .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading)
```

Your Bevy state needs to be added to the application before you can add a loading state.

You can add collections to a loading state in multiple places (e.g. in different plugins). All collections added anywhere in your application will be loaded. Important is, that the loading state itself is added to the application before you try to add any collections to it.

## Compile time vs. Run time (dynamic) assets

Asset configurations, like their file path or dimensions of sprite sheets, can be given at compile time (through derive macro attributes), or at run time (["Dynamic assets"](#dynamic-assets)). The second, allows managing asset configurations as assets. That means you can keep a list of your asset files and their properties in asset files. The main benefit of using dynamic assets is a cleaner split of code and data leading to less recompiles while working on your assets. It also makes your game more approachable for people that want to contribute without touching code.

The derive macro for `AssetCollection` supports multiple attributes. They configure how the asset is loaded.

The following code sets up a loading state with a collection that has all it's configuration in derive macro attributes:
```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::AssetLoading)
        .add_systems(OnEnter(GameState::Next), use_my_assets)
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "walking.ogg")]
    walking: Handle<AudioSource>,
}

fn use_my_assets(_my_assets: Res<MyAssets>) {
    // do something using the asset handles from the resource
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    Next,
}
```

The [full_collection](bevy_asset_loader/examples/full_collection.rs) example showcases all the different kinds of fields that an asset collection can contain using only derive macro attributes.

### Dynamic assets

Dynamic assets are configured through the derive macro attribute `key` and are not allowed to have a `path` or `paths` attribute:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct ImageAssets {
  #[asset(key = "player")]
  player: Handle<Image>,
  #[asset(key = "tree")]
  tree: Handle<Image>,
}
```

The keys `player` and `tree` in the example above should either be set manually in the `DynamicAssets` resource prior to the loading state (see the [manual_dynamic_asset](bevy_asset_loader/examples/manual_dynamic_asset.rs) example), or be part of a dynamic assets file (see [dynamic_asset](bevy_asset_loader/examples/dynamic_asset.rs)). A dynamic assets file for the collection above might look like this:
```ron
({
    "player": File (
        path: "images/player.png",
    ),
    "tree": File (
        path: "images/tree.png",
    ),
})
```

Using dynamic assets like `File` and loading ron files requires the `standard_dynamic_assets` feature to be enabled.

The file ending is `.assets.ron` by default, but can be configured via `LoadingState::set_standard_dynamic_asset_collection_file_endings`.

Dynamic assets can be optional. This requires the derive attribute `optional` on the field and the type to be an `Option`. The value of the field will be `None` in case the given key cannot be resolved at run time.

The example [full_dynamic_collection](bevy_asset_loader/examples/full_dynamic_collection.rs) shows all supported field types for dynamic assets. Note that adding a dynamic asset file to a loading state requires the `AssetServer` resource to be available. In most cases that means that you should add the `DefaultPlugins` before configuring your loading state.

### Custom dynamic assets

You can define your own types to load as dynamic assets. Take a look at the [custom_dynamic_assets.rs](bevy_asset_loader/examples/custom_dynamic_assets.rs) example for some code.

## Supported asset fields

The simplest field is of the type `Handle<T>` and is loaded from a single file without any special processing. One example might be audio sources, but any asset type that has an asset loader registered with Bevy can be used like this.

The field should only have the `path` attribute set. The path is relative to your `assets` directory.
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "my-background.ogg")]
    background: Handle<AudioSource>,
}
```

The dynamic version of the same collection looks like this:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(key = "background")]
    background: Handle<AudioSource>,
}
```
```ron
({
    "background": File (
        path: "my-background.ogg",
    ),
})
```


The following sections describe more types of asset fields that you can load through asset collections.

### Collections

#### Folders

*This asset field type is not supported in web builds. See [Files](#list-of-paths) for a web compatible way of loading a collection of files.*

You can load all files in a folder as a vector of untyped handles. This field requires the additional derive macro attribute `collection`:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "images", collection)]
    folder: Vec<HandleUntyped>,
}
```

Just like Bevy's `load_folder`, this will also recursively load sub folders.

If all assets in the folder have the same (known) type, you can load the folder as `Vec<Handle<T>>` by setting `typed` in the `collection` attribute. Don't forget to adapt the type of the struct field:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "images", collection(typed))]
    folder: Vec<Handle<Image>>,
}
```

Folders are also supported as a dynamic asset. The path attribute is replaced by the `key` attribute:
```rust ignore
#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(key = "my.images", collection(typed))]
    images: Vec<Handle<Image>>,
}
```
```ron
({
    "my.images": Folder (
        path: "images",
    ),
})
```

Loading folders is not supported for web builds. If you want to be compatible with Wasm, load you handles from a list of paths instead (see next section).

#### List of paths

If you want to load a list of asset files with the same type into a vector of `Handle<T>`, you can list their paths in an attribute:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    files_typed: Vec<Handle<Image>>,
}
```

In case you do not know their types, or they might have different types, the handles can also be untyped:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(paths("images/player.png", "sound/background.ogg"), collection)]
    files_untyped: Vec<HandleUntyped>,
}
```

As dynamic assets, these two fields replace their `paths` attribute with `key`. This is the same as for folders.
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(key = "files_untyped", collection)]
    files_untyped: Vec<HandleUntyped>,
    #[asset(key = "files_typed", collection(typed))]
    files_typed: Vec<Handle<Image>>,
}
```

The corresponding assets file differs from the folder example:
```ron
({
    "files_untyped": Files (
        paths: ["images/tree.png", "images/player.png"],
    ),
    "files_typed": Files (
        paths: ["images/tree.png", "images/player.png"],
    ),
})
```

#### Collections as maps

Collections can be loaded as maps using their file paths as the keys. This is only a change in derive attributes and asset field type. Some examples from the sections above would look like this:

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "images", collection(mapped))]
    folder: HashMap<String, HandleUntyped>,
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed, mapped))]
    files_typed: HashMap<String, Handle<Image>>,
    #[asset(key = "files_untyped", collection(mapped))]
    dynamic_files_untyped: HashMap<String, HandleUntyped>,
    #[asset(key = "files_typed", collection(typed, mapped))]
    dynamic_files_typed: HashMap<String, Handle<Image>>,
}
```

### Images

Asset collections support configuring the sampler of an image asset through a derive attribute. You can configure either the sampler like so:

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler = linear))]
    tree_linear: Handle<Image>,

    #[asset(path = "images/pixel_tree.png")]
    #[asset(image(sampler = nearest))]
    tree_nearest: Handle<Image>,
}
```

### Standard materials

You can directly load standard materials if you enable the feature `3d`. For a complete example please take a look at [standard_material.rs](bevy_asset_loader/examples/standard_material.rs).

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(standard_material)]
    #[asset(path = "images/player.png")]
    player: Handle<StandardMaterial>,
}
```

This is also supported as a dynamic asset:
```rust ignore
#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(key = "image.player")]
    player: Handle<StandardMaterial>,
}
```
```ron
({
    "image.player": StandardMaterial (
        path: "images/player.png",
    ),
})
```

### Texture atlases

You can directly load texture atlases from sprite sheets if you enable the feature `2d`. For a complete example please take a look at [atlas_from_grid.rs](bevy_asset_loader/examples/atlas_from_grid.rs).

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 8, rows = 1, padding_x = 12., padding_y = 12., offset_x = 6., offset_y = 6.))]
    #[asset(path = "images/sprite_sheet.png")]
    sprite: Handle<TextureAtlas>,
}
```

As a dynamic asset this example becomes:
```rust ignore
#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(key = "image.player")]
    sprite: Handle<TextureAtlas>,
}
```
```ron
({
    "image.player": TextureAtlas (
        path: "images/sprite_sheet.png",
        tile_size_x: 100.,
        tile_size_y: 64.,
        columns: 8,
        rows: 1,
        padding_x: 12.,
        padding_y: 12.,
        offset_x: 6.,
        offset_y: 6.,
    ),
})
```

The four padding & offset fields/attributes are optional, and default to `0.`.

### Types implementing FromWorld

Any field in an asset collection without any attribute is required to implement the `FromWorld` trait. When the asset collection is build, the `FromWorld` implementation is called to get the value for the field.

## Initializing FromWorld resources

In situations where you would like to prepare other resources based on your loaded asset collections you can use `App::init_resource_after_loading_state` to initialize `FromWorld` resources. See [init_resource.rs](bevy_asset_loader/examples/init_resource.rs) for an example that loads two images and then combines their pixel data into a third image.

`App::init_resource_after_loading_state` does the same as Bevy's `App::init_resource`, but at a different point in time. While Bevy inserts your resources at the very beginning, `bevy_asset_loader` will initialize them only after your loaded asset collections are inserted. That means you can use your asset collections in the `FromWorld` implementation.

## Progress tracking

With the feature `progress_tracking`, you can integrate with [`iyes_progress`][iyes_progress] to track asset loading during a loading state. This, for example, enables progress bars.

See [`progress_tracking`](bevy_asset_loader/examples/progress_tracking.rs) for a complete example.

### A note on system ordering

The loading state is organized in a private schedule that runs in a single system during the `Update` schedule. If you want to explicitly order against the system running the loading state, you can do so with the system set `LoadingStateSet`.

## Failure state

You can configure a failure state in case some asset in a collection fails to load by calling `on_failure_continue_to` with a state (see [`failure_state`](bevy_asset_loader/examples/failure_state.rs) example). If no failure state is configured and some asset fails to load, your application will be stuck in the loading state.

In most cases this happens, an asset file is missing or a certain file ending does not have a corresponding asset loader. In both of these cases the application log should help since Bevy prints warnings about those issues.

## Usage without a loading state

Although the pattern of a loading state is quite nice, you might have reasons not to use it. In this case `bevy_asset_loader` can still be helpful. Deriving `AssetCollection` on a resource can significantly reduce the boilerplate for managing assets.

You can directly initialise asset collections on the bevy `App` or `World`. See [no_loading_state.rs](bevy_asset_loader/examples/no_loading_state.rs) for a complete example.

```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_collection::<MyAssets>()
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 96., columns = 8, rows = 1, padding_x = 12., padding_y = 12.))]
    #[asset(path = "images/sprite_sheet.png")]
    sprite: Handle<TextureAtlas>,
}
```

## Unloading assets

Bevy unloads an asset when there are no strong asset handles left pointing to the asset. An `AssetCollection` stores strong handles and ensures that assets contained in it are not removed from memory. If you want to unload assets, you need to remove any `AssetCollection` resource that holds handles pointing to those assets. You, for example, could do this when leaving the state that needed the collection.

## Compatible Bevy versions

The main branch is compatible with the latest Bevy release, while the branch `bevy_main` tries to track the `main` branch of Bevy (PRs updating the tracked commit are welcome).

Compatibility of `bevy_asset_loader` versions:
| `bevy_asset_loader` | `bevy` |
| :--                 | :--    |
| `0.17`              | `0.11` |
| `0.15` - `0.16`     | `0.10` |
| `0.14`              | `0.9`  |
| `0.12` - `0.13`     | `0.8`  |
| `0.10` - `0.11`     | `0.7`  |
| `0.8` - `0.9`       | `0.6`  |
| `0.1` - `0.7`       | `0.5`  |
| branch `main`       | `0.11` |
| branch `bevy_main`  | `main` |

## License

Dual-licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](/LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](/LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Assets in the examples might be distributed under different terms. See the [readme](bevy_asset_loader/examples/README.md#credits) in the `bevy_asset_loader/examples` directory.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.



[bevy]: https://bevyengine.org/
[cheatbook-states]: https://bevy-cheatbook.github.io/programming/states.html
[iyes_progress]: https://github.com/IyesGames/iyes_progress
