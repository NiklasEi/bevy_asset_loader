# Changelog

- add support for dynamic asset paths ([#14](https://github.com/NiklasEi/bevy_asset_loader/issues/14))
- move functionality behind `sprite` feature to `render` feature
  - both features require the bevy feature `render`
- support loading a folder in an asset collection ([#19](https://github.com/NiklasEi/bevy_asset_loader/issues/19))
- remove `render` and `sprite` features from the default features
- allow using the plugin without a "next" state ([#20](https://github.com/NiklasEi/bevy_asset_loader/issues/20))
  - `AssetLoader::new` now only takes the "loading" state
  - an optional "next" state can be configured via `AssetLoader::continue_to_state`  

## v0.6.0
- support loading asset collections with padding ([#10](https://github.com/NiklasEi/bevy_asset_loader/issues/10))
- fix feature resolving for loading texture atlases ([#12](https://github.com/NiklasEi/bevy_asset_loader/issues/12))
- better compiler errors for derive macro
- make `AssetCollection::load` more flexible by taking the ECS world instead of only the AssetServer ([#14](https://github.com/NiklasEi/bevy_asset_loader/issues/14))
- support loading color materials ([#13](https://github.com/NiklasEi/bevy_asset_loader/issues/13))
