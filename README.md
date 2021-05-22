# Bevy asset loader

*WIP: already functional, but lacking some features. Feedback very welcome!*

[![crates.io](https://img.shields.io/crates/v/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)
[![docs](https://docs.rs/bevy_asset_loader/badge.svg)](https://docs.rs/bevy_asset_loader)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/NiklasEi/bevy_asset_loader/blob/main/LICENSE.md)
[![crates.io](https://img.shields.io/crates/d/bevy_asset_loader.svg)](https://crates.io/crates/bevy_asset_loader)

This [Bevy][bevy] plugin reduces boilerplate when loading game assets. The crate offers the `AssetCollection` trait and can automatically load structs that implement it. The trait can be derived.

# How to use
The plugin takes an AssetCollection and two states as configuration. During the first stage it will load the assets and insert the struct implementing `AssetCollection` as a resource. When the assets are done loading, the plugin switches to the second state.

```rust
fn main() {
    App::build()
        .add_state(MyStates::AssetLoading)
        .add_plugin(AssetLoaderPlugin::<MyAudioAssets, _>::new(
            MyStates::AssetLoading,
            MyStates::Next,
        ))
        .add_system_set(
            SystemSet::on_enter(MyStates::Next).with_system(use_my_audio_assets.system()),
        )
        .run();
}

#[derive(AssetCollection)]
struct MyAudioAssets {
    #[asset(path = "audio/walking.ogg")]
    walking: Handle<AudioSource>,
    #[asset(path = "audio/flying.ogg")]
    flying: Handle<AudioSource>,
}

fn use_my_audio_assets(_assets: Res<MyAudioAssets>) {
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
