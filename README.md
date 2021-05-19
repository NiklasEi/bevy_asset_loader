# Bevy asset loader

*WIP: already functional, but lacking some features. Feedback very welcome!*

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

fn use_my_audio_assets(assets: Res<MyAudioAssets>) {
    // do something using the asset handles in `assets`
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}
```


[bevy]: https://bevyengine.org/
