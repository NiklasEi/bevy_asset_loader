# Bevy asset loader

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/crates/l/bevy_asset_loader)](https://github.com/NiklasEi/bevy_asset_loader#license)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

_Loading states with minimal boilerplate_

This [Bevy][bevy] plugin reduces boilerplate for handling game assets. The crate offers the derivable `AssetCollection` trait and can automatically load structs that implement it. Asset collections contain handles to your game assets and are available to your systems as resources after loading.

In most cases you will want to load your asset collections during loading states (think loading screens). During such a state, all assets are loaded and their loading progress is observed. Only when asset collections can be built with fully loaded asset handles, the collections are inserted to Bevy's ECS as resources. If you do not want to use a loading state, asset collections can still result in cleaner code and improved maintainability (see the ["usage without a loading state"](#usage-without-a-loading-state) section).

_The `main` branch and the latest release support Bevy version `0.14` (see [version table](#compatible-bevy-versions))_

## Loading states

A loading state is responsible for managing the loading process during a configurable Bevy state (see [the cheatbook on states][cheatbook-states]).

If your `LoadingState` is set up, you can start your game logic from the configured "next state" and use the asset collections as resources in your systems. The loading state guarantees that all handles in your collections are fully loaded by the time the next state starts.

```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<AudioAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), start_background_audio)
        .run();
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

/// This system runs in MyStates::Next. Thus, AudioAssets is available as a resource
/// and the contained handle is done loading.
fn start_background_audio(mut commands: Commands, audio_assets: Res<AudioAssets>) {
    commands.spawn(AudioBundle {
        source: audio_assets.background.clone(),
        settings: PlaybackSettings::LOOP,
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
```

If we want to add additional asset collections to the just added loading state, we can do so using `configure_loading_state` at any point in our application. We could, for example, add a `PlayerPlugin` to the `App` after adding the loading state. The `PlayerPlugin` will contain all things "player" including its sprite.

```rust
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_loading_state(
                LoadingStateConfig::new(MyStates::AssetLoading)
                    .load_collection::<PlayerAssets>(),
            );
    }
}

#[derive(AssetCollection, Resource)]
struct PlayerAssets {
    #[asset(path = "images/player.png")]
    sprite: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
```

## Compile time vs. Run time (dynamic) assets

Asset configurations, like their file path or dimensions of sprite sheets, can be given at compile time (through derive macro attributes), or at run time (["Dynamic assets"](#dynamic-assets)). The second, allows managing asset configurations as assets. That means you can keep a list of your asset files and their properties in asset files. The main benefit of using dynamic assets is a cleaner split of code and data leading to less recompiles while working on your assets. It also makes your game more approachable for people that want to contribute without touching code.

The derive macro for `AssetCollection` supports multiple attributes. They configure how the asset is loaded.

### Compile time assets

The two earlier code examples show a loading state with collections that have all their configuration in derive macro attributes. The path of the player sprite is hardcoded to be "images/player.png". Changing it will require recompiling your application.

The [full_collection](/bevy_asset_loader/examples/full_collection.rs) example showcases all the different kinds of fields that an asset collection can contain using only derive macro attributes.

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

The keys `player` and `tree` in the example above should either be set manually in the `DynamicAssets` resource prior to the loading state
(see the [manual_dynamic_asset](/bevy_asset_loader/examples/manual_dynamic_asset.rs) example), or be part of a dynamic assets file (see [dynamic_asset.rs](/bevy_asset_loader/examples/dynamic_asset.rs)).
A dynamic assets file for the collection above might look like this:

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

The example [full_dynamic_collection](/bevy_asset_loader/examples/full_dynamic_collection.rs) shows all supported field types for dynamic assets. Note that adding a dynamic asset file to a loading state requires the `AssetServer` resource to be available. In most cases that means that you should add the `DefaultPlugins` before configuring your loading state.

### Custom dynamic assets

You can define your own types to load as dynamic assets. Take a look at the [custom_dynamic_assets.rs](/bevy_asset_loader/examples/custom_dynamic_assets.rs) example for some code.

## Supported asset fields

The simplest field is of the type `Handle<T>` and is loaded from a single file. One example might be audio sources, but any asset type that has an asset loader registered with Bevy can be used like this.

The field should only have the `path` attribute set.

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

### Texture atlases

You can create texture atlas layouts as part of an `AssetCollection` if you enable the feature `2d`. For a complete example please take a look at [atlas_from_grid.rs](/bevy_asset_loader/examples/atlas_from_grid.rs).

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 8, rows = 1, padding_x = 12, padding_y = 12, offset_x = 6, offset_y = 6))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "images/sprite_sheet.png")]
    sprite: Handle<Image>,
}
```

As a dynamic asset this example becomes:

```rust ignore
#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(key = "player.layout")]
    layout: Handle<TextureAtlasLayout>,
    #[asset(key = "player.image")]
    sprite: Handle<Image>,
}
```

```ron
({
    "player.image": File (
        path: "images/sprite_sheet.png",
    ),
    "player.layout": TextureAtlasLayout (
        tile_size_x: 100,
        tile_size_y: 64,
        columns: 8,
        rows: 1,
        padding_x: 12,
        padding_y: 12,
        offset_x: 6,
        offset_y: 6,
    ),
})
```

The four padding & offset fields/attributes are optional, and default to `0`.

### Images with sampler configuration

Asset collections support configuring the sampler of an image asset through a derive attribute:

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

The corresponding dynamic asset would be

```ron
({
    "tree_nearest": Image (
        path: "images/tree.png",
        sampler: Nearest
    ),
    "tree_linear": Image (
        path: "images/tree.png",
        sampler: Linear
    ),
})
```

### Array images

You can let `bevy_asset_loader` configure the layers of a texture array.

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/array_texture.png")]
    #[asset(image(array_texture_layers = 4))]
    array_texture: Handle<Image>,
}
```

The corresponding dynamic asset would be

```ron
({
    "array_texture": Image (
        path: "images/array_texture.png",
        array_texture_layers: 4
    ),
})
```

### Standard materials

You can directly load standard materials if you enable the feature `3d`. For a complete example please take a look at [standard_material.rs](/bevy_asset_loader/examples/standard_material.rs).

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

### Collections

#### Folders

_This asset field type is not supported in web builds. See [Files](#list-of-paths) for a web compatible way of loading a collection of files._

You can load all files in a folder as a vector of untyped handles. This field requires the additional derive macro attribute `collection`:

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "images", collection)]
    folder: Vec<UntypedHandle>,
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
    files_untyped: Vec<UntypedHandle>,
}
```

As dynamic assets, these two fields replace their `paths` attribute with `key`. This is the same as for folders.

```rust
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(key = "files_untyped", collection)]
    files_untyped: Vec<UntypedHandle>,
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

### Collections as maps

Collections can be loaded as maps using any type that implements [`MapKey`](https://docs.rs/bevy_asset_loader/latest/bevy_asset_loader/mapped/trait.MapKey.html) as the keys (see documentation for more details).
This is only a change in derive attributes and asset field type. The examples from the sections above would look like this:

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::asset_collection::AssetCollection;

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "images", collection(mapped))]
    folder: HashMap<String, UntypedHandle>,
    #[asset(paths("images/player.png", "images/tree.png"), collection(typed, mapped))]
    files_typed: HashMap<String, Handle<Image>>,
    #[asset(key = "files_untyped", collection(mapped))]
    dynamic_files_untyped: HashMap<String, UntypedHandle>,
    #[asset(key = "files_typed", collection(typed, mapped))]
    dynamic_files_typed: HashMap<String, Handle<Image>>,
}
```

### Types implementing FromWorld

Any field in an asset collection without any attribute is required to implement the `FromWorld` trait. When the asset collection is build, the `FromWorld` implementation is called to get the value for the field.

## Initializing FromWorld resources

In situations where you would like to prepare other resources based on your loaded asset collections you can use `LoadingState::init_resource` or `LoadingStateConfig::init_resource` to initialize `FromWorld` resources. See [init_resource.rs](/bevy_asset_loader/examples/init_resource.rs) for an example that loads two images and then combines their pixel data into a third image.

Both `init_resource` methods from `bevy_asset_loader` do the same as Bevy's `App::init_resource`, but at a different point in time. While Bevy inserts your resources at application startup, `bevy_asset_loader` will initialize them only after your asset collections are available. That means you can use your asset collections in the `FromWorld` implementation of the resource.

## Progress tracking

With the feature `progress_tracking`, you can integrate with [`iyes_progress`][iyes_progress] to track asset loading during a loading state. This, for example, enables progress bars.

See [`progress_tracking`](/bevy_asset_loader/examples/progress_tracking.rs) for a complete example.

### A note on system ordering

The loading state is organized in a private schedule that runs in a single system during the `Update` schedule. If you want to explicitly order against the system running the loading state, you can do so with the exported system set `LoadingStateSet`.

## Failure state

You can configure a failure state in case some asset in a collection fails to load by calling `on_failure_continue_to` with a state (see the [`failure_state.rs`](/bevy_asset_loader/examples/failure_state.rs) example). If no failure state is configured and some asset fails to load, your application will be stuck in the loading state.

In most cases of failed loading states, an asset file is missing or a certain asset does not have an asset loader registered. In both of these cases, the application log should help since Bevy prints warnings about those issues.

## Usage without a loading state

Although the pattern of a loading state is quite nice (imo), you might have reasons not to use it. In this case, `bevy_asset_loader` can still be helpful. Deriving `AssetCollection` on a resource can significantly reduce the boilerplate for managing assets.

Asset collections loaded without a loading state do not support folders, dynamic assets, or the `iamge` annotation. This is because these features require some form of waiting (see [potential future support for these features](https://github.com/NiklasEi/bevy_asset_loader/issues/230)).

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

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 8, rows = 1, padding_x = 12, padding_y = 12, offset_x = 6, offset_y = 6))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "images/sprite_sheet.png")]
    sprite: Handle<Image>,
}
```

## Unloading assets

Bevy unloads an asset when there are no strong asset handles left pointing to the asset. An `AssetCollection` stores strong handles and ensures that assets contained in it are not removed from memory. If you want to unload assets, you need to remove any `AssetCollection` resource that holds handles pointing to those assets. You, for example, could do this when leaving the state that needed the collection.

## Compatible Bevy versions

The main branch is compatible with the latest Bevy release, while the branch `bevy_main` tries to track the `main` branch of Bevy (PRs updating the tracked commit are welcome).

Compatibility of `bevy_asset_loader` versions:

| Bevy version | `bevy_asset_loader` version |
|:-------------|:----------------------------|
| `0.14`       | `0.21`                      |
| `0.13`       | `0.20`                      |
| `0.12`       | `0.18` - `0.19`             |
| `0.11`       | `0.17`                      |
| `0.10`       | `0.15` - `0.16`             |
| `0.9`        | `0.14`                      |
| `0.8`        | `0.12` - `0.13`             |
| `0.7`        | `0.10` - `0.11`             |
| `0.6`        | `0.8` - `0.9`               |
| `0.5`        | `0.1` - `0.7`               |
| `0.13`       | branch `main`               |
| `main`       | branch `bevy_main`          |

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
