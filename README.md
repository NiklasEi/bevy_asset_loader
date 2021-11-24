# Bevy asset loader

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/crates/l/bevy_asset_loader)](https://github.com/NiklasEi/bevy_asset_loader/blob/main/LICENSE.md)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

This [Bevy][bevy] plugin reduces boilerplate when loading game assets. The crate offers the `AssetCollection` trait and can automatically load structs that implement it. The trait can be derived.

*The `main` branch and all current releases support Bevy version 0.5. If you like living on the edge, take a look at the `bevy_main` branch, which tries to stay close to Bevy's development.*

## How to use

The `AssetLoader` is constructed with two states (see [the cheatbook on states][cheatbook-states]). During the first state it will load the assets and check up on the loading status in every frame. When the assets are done loading, the collections will be inserted as resources, and the plugin switches to the second state.

For structs with named fields that are either asset handles or implement default, `AssetCollection` can be derived. You can add as many `AssetCollection`s to the loader as you want by chaining `with_collection` calls. To finish the setup, call the `build` function with your `AppBuilder`.

Now you can start your game logic from the second configured state and use the asset collections as resources in your systems.

```rust no_run
use bevy::prelude::*;
use bevy_asset_loader::{AssetLoader, AssetCollection};

fn main() {
  let mut app = App::build();
  AssetLoader::new(GameState::AssetLoading, GameState::Next)
          .with_collection::<TextureAssets>()
          .with_collection::<AudioAssets>()
          .build(&mut app);
  app.add_state(GameState::AssetLoading)
          .add_plugins(DefaultPlugins)
          .add_system_set(SystemSet::on_enter(GameState::Next).with_system(use_my_assets.system()))
          .run();
}

#[derive(AssetCollection)]
struct AudioAssets {
  #[asset(path = "walking.ogg")]
  walking: Handle<AudioSource>
}

#[derive(AssetCollection)]
struct TextureAssets {
  #[asset(path = "textures/player.png")]
  player: Handle<Texture>,
  #[asset(path = "textures/tree.png")]
  tree: Handle<Texture>,
}

fn use_my_assets(_texture_assets: Res<TextureAssets>, _audio_assets: Res<AudioAssets>) {
  // do something using the asset handles from the resources
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
  AssetLoading,
  Next,
}
```

See [two_collections.rs](/bevy_asset_loader/examples/two_collections.rs) for a complete example.

### Initialize FromWorld resources

In situations where you would like to prepare other resources based on your loaded assets you can use `AssetLoader::init_resource` to initialize `FromWorld` resources. See [init_resource.rs](/bevy_asset_loader/examples/init_resource.rs) for an example that loads two textures and then combines their image data into a third texture.

`AssetLoader::init_resource` does the same as Bevy's `App::init_resource`, but at a different point in time. While Bevy inserts your resources at the very beginning, the AssetLoader will do so after having inserted your loaded asset collections. That means that you can use your asset collections in the `FromWorld` implementations.

### Loading color materials

You can directly load color materials. For a complete example please take a look at [color_material.rs](/bevy_asset_loader/examples/color_material.rs).
```rust ignore
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(color_material)]
    #[asset(path = "textures/player.png")]
    player: Handle<ColorMaterial>,
}
```
*This requires the feature `sprite` which is part of the default features*

### Loading texture atlases

You can directly load texture atlases from sprite sheets. For a complete example please take a look at [atlas_from_grid.rs](/bevy_asset_loader/examples/atlas_from_grid.rs).
```rust ignore
#[derive(AssetCollection)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 100., tile_size_y = 96., columns = 8, rows = 1, padding_x = 12., padding_y = 12.))]
    #[asset(path = "textures/sprite_sheet.png")]
    sprite: Handle<TextureAtlas>,
}
```
*This requires the feature `render` which is part of the default features*

## Development

The integration tests cannot run with default features enabled. Because they do not insert the render plugin of Bevy, they fail due to the derive macro requiring the `TextureAtlas` asset collection (this is toggled with the `render` feature).  
Use `cargo test --no-default-features` to run all tests.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Assets in the examples might be distributed under different terms. See the [readme](bevy_asset_loader/examples/README.md#credits) in the `bevy_asset_loader/examples` directory.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

[bevy]: https://bevyengine.org/
[cheatbook-states]: https://bevy-cheatbook.github.io/programming/states.html
