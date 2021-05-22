# Bevy asset loader

*WIP: already functional, but lacking some features. Feedback very welcome!*

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/NiklasEi/bevy_asset_loader/blob/main/LICENSE.md)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

This [Bevy][bevy] plugin reduces boilerplate when loading game assets. The crate offers the `AssetCollection` trait and can automatically load structs that implement it. The trait can be derived.

# How to use
The plugin takes an `AssetCollection` and two `State`s as configuration. During the first state it will load the assets and check up on the loading status in every frame. When the assets are done loading, the asset collection is inserted as a resource. Then the plugin switches to the second state. You can now use the assets by requesting the resource in your systems.

```rust
fn main() {
    App::build()
        .add_state(MyStates::AssetLoading)
        .add_plugin(AssetLoaderPlugin::<MyAssets, _>::new(
            MyStates::AssetLoading,
            MyStates::Next,
        ))
        .add_system_set(
            SystemSet::on_enter(MyStates::Next).with_system(use_my_assets.system()),
        )
        .run();
}

#[derive(AssetCollection)]
struct MyAssets {
    #[asset(path = "textures/player.ogg")]
    player: Handle<Texture>,
    #[asset(path = "textures/tree.ogg")]
    tree: Handle<Texture>,
}

fn use_my_assets(_assets: Res<MyAssets>) {
    // do something using the asset handles in `assets`
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
```

# Missing features

* Loading multiple `AssetCollection`s per plugin (see [#1](https://github.com/NiklasEi/bevy_asset_loader/issues/1))
* More than one plugin loading in the same State (maybe an alternative to the point above) (see [#2](https://github.com/NiklasEi/bevy_asset_loader/issues/2))
  * The current problem here is that the different plugins will enter a race for who is allowed to change the State first


[bevy]: https://bevyengine.org/
