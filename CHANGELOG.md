# Changelog

- Support loading folders as vectors of typed handles ([#30](https://github.com/NiklasEi/bevy_asset_loader/issues/30))
- 'Folder' can be used as alias for 'File' in dynamic asset collection ron files

## v0.9.0
- Folder assets are now marked with the `folder` attribute. The path is defined as for any other asset in `path`.
  - This adds support for dynamic folders in asset collections ([#28](https://github.com/NiklasEi/bevy_asset_loader/issues/28))
- Support optional dynamic assets in collections ([#24](https://github.com/NiklasEi/bevy_asset_loader/issues/24))
- Improved compile error if the `texture_atlas` or `standard_material` attributes are used without the render feature (related to [#27](https://github.com/NiklasEi/bevy_asset_loader/issues/27))
- Support loading keys for dynamic assets from ron files
  - New example `dynamic_asset_ron`
- Support initialising asset collections directly on the bevy App or World
  - New example `no_loading_state`
- rename derive attribute `color_material` to `standard_material`

## v0.8.0
- update to Bevy version 0.6

## v0.7.0
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
