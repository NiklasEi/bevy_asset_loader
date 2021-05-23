# Bevy asset loader

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/NiklasEi/bevy_asset_loader/blob/main/LICENSE.md)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

This [Bevy][bevy] plugin reduces boilerplate when loading game assets. The crate offers the `AssetCollection` trait and can automatically load structs that implement it. The trait can be derived.

# How to use

The `AssetLoader` is constructed with two `State`s. During the first state it will load the assets and check up on the loading status in every frame. When the assets are done loading, the collections will be inserted as resources, and the plugin switches to the second state.

You can add as many `AssetCollection`s to the loader as you want. This is done by chaining `with_collection` calls. To finish the setup, call the `build` function with your `AppBuilder`.

Now you can start your game logic from the second configured state and use the `AssetCollection`s as resources in your systems.

```rust
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
  #[asset(path = "background.ogg")]
  background: Handle<AudioSource>
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

See the [example](/bevy_asset_loader/examples/two_collections.rs) for a complete setup. 

[bevy]: https://bevyengine.org/
