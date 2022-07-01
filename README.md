# Bevy asset loader

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/crates/l/bevy_asset_loader)](https://github.com/NiklasEi/bevy_asset_loader#license)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

This [Bevy][bevy] plugin reduces boilerplate for handling game assets. The crate offers the derivable `AssetCollection` trait and can automatically load structs that implement it. Asset collections contain handles to your game assets and are available to your systems as resources after loading.

In most cases you will want to load your asset collections during loading states (think loading screens). During such a state, all assets are loaded and their loading process is observed. Only when asset collections can be build with fully loaded asset handles, the collections are inserted as resources. If you do not want to use a loading state, asset collections can still result in cleaner code and improved maintainability (see the ["usage without a loading state"](#usage-without-a-loading-state) section).

Asset configurations, like their file path or dimensions of sprite sheets, can be given at compile time (through derive macro attributes), or at run time (see ["Dynamic assets"](#dynamic-assets)). The second, allows managing asset configurations as assets. That means you can keep a list of your asset files and their properties in asset files. The main benefit of using dynamic assets is a cleaner split of code and data leading to less recompiles while working on your assets. It also makes your game more approachable for people that want to contribute without touching code.

_`bevy_asset_loader` supports `iyes_loopless` states with the [`stageless`](#stageless) feature._

_The `main` branch and the latest release support Bevy version `0.7` (see [version table](#compatible-bevy-versions))_

## Loading states

A loading state is responsible for managing the loading process during a configurable Bevy state (see [the cheatbook on states][cheatbook-states]).

If your loading state is set up, you can start your game logic from the next state and use the asset collections as resources in your systems. The `LoadingState` guarantees that all handles in your collections are fully loaded by the time the second state starts.

## Compile time vs. Run time (dynamic) assets

The derive macro for `AssetCollection` supports multiple attributes. They configure how the asset is loaded.

The following code sets up a loading state with a collection that has all it's configuration in derive macro attributes:
```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .with_collection::<MyAssets>()
        )
        .add_state(GameState::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(use_my_assets))
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "walking.ogg")]
    walking: Handle<AudioSource>,
}

fn use_my_assets(_my_assets: Res<MyAssets>) {
    // do something using the asset handles from the resource
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Next,
}
```

The [full_collection](/bevy_asset_loader/examples/full_collection.rs) example showcases all the different kinds of fields that an asset collection can contain using only derive macro attributes.

### Dynamic assets

It is possible to decide asset configurations at run time. This is done via the resource `DynamicAssets` which is a map of asset keys to their configurations. During set up of a loading state, the resource is initialized. It is later read to resolve asset keys while loading collections.

Dynamic assets are configured through the derive macro attribute `key` and are not allowed to have a `path` or `paths` attribute:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
struct ImageAssets {
  #[asset(key = "player")]
  player: Handle<Image>,
  #[asset(key = "tree")]
  tree: Handle<Image>,
}
```

The keys `player` and `tree` in the example above should either be set manually in the `DynamicAssets` resource prior to the loading state (see the [manual_dynamic_asset](/bevy_asset_loader/examples/manual_dynamic_asset.rs) example), or be part of a dynamic assets file (see [dynamic_asset](/bevy_asset_loader/examples/dynamic_asset.rs)). A dynamic assets file for the collection above might look like this:
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

The file ending is `.assets` by default, but can be configured via `LoadingState::set_dynamic_asset_collection_file_endings`.

The example [full_dynamic_collection](/bevy_asset_loader/examples/full_dynamic_collection.rs) shows all supported field types for dynamic assets.

## Supported asset fields

The simplest field is of the type `Handle<T>` and is loaded from a single file without any special processing. One example might be audio sources, but any asset type that has an asset loader registered with Bevy can be used like this.

The field should only have the `path` attribute set. The path is relative to your `assets` directory.
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "my-background.ogg")]
    background: Handle<AudioSource>,
}
```

The dynamic version of the same collection looks like this:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
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

### Folders

You can load all files in a folder as a vector of untyped handles. This field requires the additional derive macro attribute `collection`:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
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

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "images", collection(typed))]
    folder: Vec<Handle<Image>>,
}
```

Folders are also supported as a dynamic asset. The path attribute is replaced by the `key` attribute:
```rust ignore
#[derive(AssetCollection)]
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

### List of paths

If you want to load a list of asset files with the same type into a vector of `Handle<T>`, you can list their paths in an attribute:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed))]
    files_typed: Vec<Handle<Image>>,
}
```

In case you do not know their types, or they might have different types, the handles can also be untyped:
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(paths("images/player.png", "sound/background.ogg"), collection)]
    files_untyped: Vec<HandleUntyped>,
}
```

As dynamic assets, these two fields replace their `paths` attribute with `key`. This is the same as for folders.
```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
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

### Standard materials

You can directly load standard materials if you enable the feature `3d`. For a complete example please take a look at [standard_material.rs](/bevy_asset_loader/examples/standard_material.rs).

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(standard_material)]
    #[asset(path = "images/player.png")]
    player: Handle<StandardMaterial>,
}
```

This is also supported as a dynamic asset:
```rust ignore
#[derive(AssetCollection)]
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

You can directly load texture atlases from sprite sheets if you enable the feature `2d`. For a complete example please take a look at [atlas_from_grid.rs](/bevy_asset_loader/examples/atlas_from_grid.rs).

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 8, rows = 1, padding_x = 12., padding_y = 12.))]
    #[asset(path = "images/sprite_sheet.png")]
    sprite: Handle<TextureAtlas>,
}
```

As a dynamic asset this example becomes:
```rust ignore
#[derive(AssetCollection)]
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
    ),
})
```

The two padding fields/attributes are optional and default to `0.`.

### Types implementing FromWorld



## Initializing FromWorld resources

In situations where you would like to prepare other resources based on your loaded asset collections you can use `LoadingState::init_resource` to initialize `FromWorld` resources. See [init_resource.rs](/bevy_asset_loader/examples/init_resource.rs) for an example that loads two images and then combines their pixel data into a third image.

`LoadingState::init_resource` does the same as Bevy's `App::init_resource`, but at a different point in time. While Bevy inserts your resources at the very beginning, `bevy_asset_loader` will initialize them only after your loaded asset collections are inserted. That means you can use your asset collections in the `FromWorld` implementation.

## Progress tracking

With the feature `progress_tracking`, you can integrate with [`iyes_progress`][iyes_progress] to track asset loading during a loading state. This, for example, enables progress bars.

See [`progress_tracking`](/bevy_asset_loader/examples/progress_tracking.rs) for a complete example.

When using `stageless` feature, you need to add `progress_tracking_stageless` feature in addition to `progress_tracking`.

### A note on system ordering

The loading state runs in a single exclusive system `at_start`. This means that any parallel system in the loading state will always run after all asset handles have been checked for their status. You can thus read the current progress in each frame in a parallel system without worrying about frame lag.

## Usage without a loading state

Although the pattern of a loading state is quite nice, you might have reasons not to use it. In this case `bevy_asset_loader` can still be helpful. Deriving `AssetCollection` on a resource can significantly reduce the boilerplate for managing assets.

You can directly initialise asset collections on the bevy `App` or `World`. See [no_loading_state.rs](/bevy_asset_loader/examples/no_loading_state.rs) for a complete example.

```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_collection::<MyAssets>()
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 96., columns = 8, rows = 1, padding_x = 12., padding_y = 12.))]
    #[asset(path = "images/sprite_sheet.png")]
    sprite: Handle<TextureAtlas>,
}
```

## Stageless support

`bavy_asset_loader` can integrate with `iyes_loopless`, which implements ideas from Bevy's [Stageless RFC](https://github.com/bevyengine/rfcs/pull/45). The integration can be enabled with the `stageless` feature.

Currently, you must initialize the `iyes_loopless` state before you initialize your `AssetLoader`. This is a limitation due to the way `iyes_loopless` works. The following is a minimal example of integrating `bevy_asset_loader` with `iyes_loopless`:

```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::*;

fn main() {
    App::new()
        .add_loopless_state(MyStates::AssetLoading)
        .add_loading_state(
          LoadingState::new(MyStates::AssetLoading)
            .continue_to_state(MyStates::Next)
            .with_collection::<AudioAssets>()
        )
        .add_plugins(DefaultPlugins)
        .add_enter_system(MyStates::Next, use_my_assets)
        .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

fn use_my_assets(_audio_assets: Res<AudioAssets>) {
  // do something using the asset handles from the resources
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
```

When using stageless with the `progress_tracking` feature, remember to also enable the `progress_tracking_stageless` feature. See [the stageless examples](/bevy_asset_loader/examples/README.md#examples-for-stageless) for different use cases with `iyes_loopless` integration.

## Compatible Bevy versions

The main branch is compatible with the latest Bevy release, while the branch `bevy_main` tries to track the `main` branch of Bevy (PRs updating the tracked commit are welcome).

Compatibility of `bevy_asset_loader` versions:
| `bevy_asset_loader` | `bevy` |
| :--                 |  :--   |
| `0.10` - `0.11`     | `0.7`  |
| `0.8` - `0.9`       | `0.6`  |
| `0.1` - `0.7`       | `0.5`  |
| `main`              | `0.7`  |
| `bevy_main`         | `main` |

## License

Dual-licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](/LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](/LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Assets in the examples might be distributed under different terms. See the [readme](/bevy_asset_loader/examples/README.md#credits) in the `bevy_asset_loader/examples` directory.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.



[bevy]: https://bevyengine.org/
[cheatbook-states]: https://bevy-cheatbook.github.io/programming/states.html
[iyes_progress]: https://github.com/IyesGames/iyes_progress
