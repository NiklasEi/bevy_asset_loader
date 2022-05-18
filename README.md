# Bevy asset loader

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/crates/l/bevy_asset_loader)](https://github.com/NiklasEi/bevy_asset_loader/blob/main/LICENSE.md)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

This [Bevy][bevy] plugin reduces boilerplate for handling game assets. The crate offers the derivable `AssetCollection` trait and can automatically load structs that implement it. These structs contain handles to your game assets. Each asset collection is available to your systems as a `Resource` after loading.

The plugin supports different paths for asset collections to be loaded. The most common one is a loading state (think loading screen). During this state, all assets are loaded. Only when all asset collections can be build with fully loaded asset handles, the collections are inserted as resources. If you do not want to use a loading state, asset collections can still result in cleaner code and improved maintainability for projects with a lot of assets (see ["Usage without a loading state"](#usage-without-a-loading-state)).

Asset configurations, like their file path or tile dimensions for sprite sheets, can be resolved at compile time (through derive macro attributes), or at run time (see ["Dynamic assets"](#dynamic-assets)). The second allows managing asset configurations as assets. This means you can keep a list of your asset files and their properties in asset files (at the moment only `ron` files are supported).

Asset loader also supports `iyes_loopless` states via [`stageless`](#stageless) feature.

_The `main` branch and the latest release support Bevy version `0.7` (see [version table](#compatible-bevy-versions)). If you like living on the edge, take a look at the `bevy_main` branch, which tries to stay close to Bevy's development._

## How to use

An `AssetLoader` is responsible for managing the loading process during a configurable loading state (see [the cheatbook on states][cheatbook-states]). A second state can be configured to move on to, when all assets are loaded and the collections were inserted as resources.

For structs with named fields that are either asset handles, implement default, or are of another supported type, `AssetCollection` can be derived. You can add as many asset collections to the loader as you want by chaining `with_collection` calls. To finish the setup, call the `build` function with your `AppBuilder`.

Now you can start your game logic from the second configured state and use the asset collections as resources in your systems. The `AssetLoader` guarantees that all handles in your collections are fully loaded at the time the second state starts.

```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};

fn main() {
  let mut app = App::new();
  AssetLoader::new(GameState::AssetLoading)
          .continue_to_state(GameState::Next)
          .with_collection::<ImageAssets>()
          .with_collection::<AudioAssets>()
          .build(&mut app);
  app.add_state(GameState::AssetLoading)
          .add_plugins(DefaultPlugins)
          .add_system_set(SystemSet::on_enter(GameState::Next).with_system(use_my_assets))
          .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
  #[asset(path = "walking.ogg")]
  walking: Handle<AudioSource>
}

#[derive(AssetCollection)]
struct ImageAssets {
  #[asset(path = "images/player.png")]
  player: Handle<Image>,
  #[asset(path = "images/tree.png")]
  tree: Handle<Image>,
}

fn use_my_assets(_image_assets: Res<ImageAssets>, _audio_assets: Res<AudioAssets>) {
  // do something using the asset handles from the resources
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
  AssetLoading,
  Next,
}
```

See [two_collections.rs](/bevy_asset_loader/examples/two_collections.rs) for a complete example.

### Dynamic assets

It is possible to decide asset configurations at run-time. This is done via the resource `DynamicAssets` which is basically a map of asset keys to their configurations. The `AssetLoader` initializes the resource and reads it during the loading state.

```rust
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
struct ImageAssets {
  #[asset(key = "player")]
  player: Handle<Image>,
  #[asset(key = "tree")]
  tree: Handle<Image>,
}
```

The key `player` in the above example should be either set manually in the `DynamicAssets` resource before the loading state (see the [dynamic_asset](/bevy_asset_loader/examples/dynamic_asset.rs) example), or should be part of a `.assets` file in ron format (supported file endings can be configured with `AssetLoader::set_dynamic_asset_collection_file_endings`):

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

Loading dynamic assets from such a ron file requires the feature `dynamic_assets` and a little setup. Take a look at the [dynamic_asset_ron](/bevy_asset_loader/examples/dynamic_asset_ron.rs) example to see what this can look like in your game.

### Loading a folder as asset

You can load all assets in a folder and keep them in an `AssetCollection` as a vector of untyped handles.

```rust
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "images", folder)]
    folder: Vec<HandleUntyped>,
}
```

Just like Bevy's `load_folder`, this will also recursively load sub folders.

If all assets in the folder have the same type you can load the folder as `Vec<Handle<T>>`. Just set `typed` in the `folder` attribute and adapt the type of the asset collection field.

```rust
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "images", folder(typed))]
    folder: Vec<Handle<Image>>,
}
```

### Loading standard materials

You can directly load standard materials if you enable the feature `render`. For a complete example please take a look at [standard_material.rs](/bevy_asset_loader/examples/standard_material.rs).

```rust
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(standard_material)]
    #[asset(path = "images/player.png")]
    player: Handle<StandardMaterial>,
}
```

This is also supported as a dynamic asset:

```ron
({
    "image.tree": StandardMaterial (
        path: "images/tree.png",
    ),
})
```

### Loading texture atlases

You can directly load texture atlases from sprite sheets if you enable the feature `render`. For a complete example please take a look at [atlas_from_grid.rs](/bevy_asset_loader/examples/atlas_from_grid.rs).

```rust
use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 64., columns = 8, rows = 1, padding_x = 12., padding_y = 12.))]
    #[asset(path = "images/sprite_sheet.png")]
    sprite: Handle<TextureAtlas>,
}
```

This is also supported as a dynamic asset:

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

### Initialize FromWorld resources

In situations where you would like to prepare other resources based on your loaded assets you can use `AssetLoader::init_resource` to initialize `FromWorld` resources. See [init_resource.rs](/bevy_asset_loader/examples/init_resource.rs) for an example that loads two images and then combines their pixel data into a third image.

`AssetLoader::init_resource` does the same as Bevy's `App::init_resource`, but at a different point in time. While Bevy inserts your resources at the very beginning, the AssetLoader will do so after having inserted your loaded asset collections. That means that you can use your asset collections in the `FromWorld` implementations.

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
use bevy_asset_loader::{AssetCollection, AssetCollectionApp};

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

## Stageless

Asset loader supports Stageless RFC via `iyes_loopless`. It can be enabled with `stageless` feature.

Note that currently you must initialize loopless state before you initialize `AssetLoader`. This is a limitation due to the way `iyes_loopless` works.

When using with `progress_tracking`, remember to enable `progress_tracking_stageless` feature too.

```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_loopless::prelude::*;

/// This example shows how to use `iyes_loopless` with `bevy_asset_loader`
fn main() {
    let mut app = App::new();
    app.add_loopless_state(GameState::AssetLoading);
    AssetLoader::new(GameState::AssetLoading)
        .continue_to_state(GameState::Next)
        .with_collection::<ImageAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    app
        .add_plugins(DefaultPlugins)
        .add_enter_system(GameState::Next, use_my_assets)
        .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection)]
struct ImageAssets {
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
    #[asset(path = "images", folder)]
    _images: Vec<HandleUntyped>,
}


fn use_my_assets(_image_assets: Res<ImageAssets>, _audio_assets: Res<AudioAssets>) {
  // do something using the asset handles from the resources
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
```

## Compatible Bevy versions

The main branch is compatible with the latest Bevy release, while the branch `bevy_main` tracks the `main` branch of Bevy.

Compatibility of `bevy_asset_loader` versions:
| `bevy_asset_loader` | `bevy` |
| :-- | :-- |
| `0.10` | `0.7` |
| `0.8` - `0.9` | `0.6` |
| `0.1` - `0.7` | `0.5` |
| `main` | `0.7` |
| `bevy_main` | `main` |

## License

Licensed under either of

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
