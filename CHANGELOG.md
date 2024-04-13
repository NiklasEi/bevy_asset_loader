# Changelog

- new map key type `AssetLabel` that creates an asset map using the label as the key

## v0.20.1 - 08.04.2024
- introduces the ability to load and build collections of dynamic assets (resolves [#186](https://github.com/NiklasEi/bevy_asset_loader/issues/186))

## v0.20.0 - 18.02.2024
- update to Bevy 0.13
- support any type implementing the new trait `MapKey` as keys for mapped assets (resolves [#153](https://github.com/NiklasEi/bevy_asset_loader/issues/153))
- derive Serialize on StandardDynamicAssets (resolves [#177](https://github.com/NiklasEi/bevy_asset_loader/issues/177))

### v0.19.1 - 03.01.2024
- Fix: `standard_dynamic_assets` feature fails compilation without `2d` or `3d` feature (resolves [#175](https://github.com/NiklasEi/bevy_asset_loader/issues/175))

## v0.19.0 - 27.12.2023 - yanked
- New API to add and configure loading states
  - Less API surface on Bevy's App type and fewer generics
    - Only two methods on the App: `add_loading_state` and `configure_loading_state`
  - The old API is still supported, but deprecated for easier migration 
- Deny unknown fields in standard dynamic assets
- Support configuring the image sampler for texture atlases

## v0.18.0 - 08.11.2023
- update to Bevy 0.12
- Make `loading_state::LoadingStateSet` public for explicit system ordering
- Support configuring an image sampler through a derive attribute ([#156](https://github.com/NiklasEi/bevy_asset_loader/pull/156))
  - See [the new example](bevy_asset_loader/examples/image_asset.rs)
  - This can is also supported in dynamic assets through the new standard dynamic asset `Image`
- Optional fields of dynamic assets no longer require wrapping their values in `Some`
  - E.g. configuring `padding_x` for a texture atlas is now just `padding_x: 1.5` instead of `padding_x: Some(1.5)`

## v0.17.0
- update to Bevy 0.11
- Use "/" in paths used as keys for mapped collections on all platforms (resolves [#135](https://github.com/NiklasEi/bevy_asset_loader/issues/135))

## v0.16.1
- Fix running loading states in parallel states, meaning using multiple state types (resolves [#125](https://github.com/NiklasEi/bevy_asset_loader/issues/125))
- Dynamic asset files can be loaded multiple times now
  - re-entering a loading state containing dynamic assets (see [#126](https://github.com/NiklasEi/bevy_asset_loader/issues/126))
  - loading dynamic assets in multiple states (see [#123](https://github.com/NiklasEi/bevy_asset_loader/issues/123))
- Fix crash with non default exit criteria (see [#121](https://github.com/NiklasEi/bevy_asset_loader/issues/121))

## v0.16.0
- Remove features coming from optional dependencies
- Allow usage of non default exit criteria; workaround for [bevy#8386](https://github.com/bevyengine/bevy/issues/8386)
- Move compile error to field with missing `FromWorld` impl

## v0.15.0
- Option to load folders as maps of path to handle (resolves [#32](https://github.com/NiklasEi/bevy_asset_loader/issues/32))
- Changed the default file ending for standard dynamic asset files from `.assets` to `.assets.ron`
- Fix: multiple calls of `with_dynamic_collections` (resolves [#99](https://github.com/NiklasEi/bevy_asset_loader/issues/99))
- Update to Bevy `0.10`
  - Removed features `stageless` and `progress_tracking_stageless`

## v0.14.1
- Make sure that the `RonAssetPlugin` for `StandardDynamicAssetCollection` is only added once

## v0.14.0
- update to Bevy `0.9`

## v0.13.0
- renamed feature `dynamic_assets` to `standard_dynamic_assets`, since you can use custom dynamic assets without the feature
- Empty loading states no longer get stuck, but directly transition to the next state (fixes [#82](https://github.com/NiklasEi/bevy_asset_loader/issues/82))
- Add support for failure states ([#79](https://github.com/NiklasEi/bevy_asset_loader/issues/79))
- bump `iyes_loopless` to `0.8`
- bump `iyes_progress` to `0.6`

## v0.12.1
- bump `iyes_progress` to `0.5`

## v0.12.0
- `App` extension trait making adding loading states a lot nicer ([#58](https://github.com/NiklasEi/bevy_asset_loader/issues/58))
- Custom dynamic assets ([#55](https://github.com/NiklasEi/bevy_asset_loader/issues/55))
- Allow configuring a loading state multiple times (fixes [#60](https://github.com/NiklasEi/bevy_asset_loader/issues/60))
- Support optional dynamic folder and files assets (fixes [#49](https://github.com/NiklasEi/bevy_asset_loader/issues/49))
- Added a prelude
- Documentation fixes and improvements
- `AssetLoader` => `LoadingState`
- Update to Bevy 0.8

## v0.11.0
- Support progress tracking through `iyes_progress` ([#6](https://github.com/NiklasEi/bevy_asset_loader/issues/6))
- Use `FromWorld` instead of `Default` for fields without attributes
  - this enables complex custom types in asset collections
- Support `iyes_loopless`, which implements ideas from Bevy's [Stageless RFC](https://github.com/bevyengine/rfcs/pull/45)
  - requires the `stageless` feature
  - note that progress tracking needs the `progress_tracking_stageless` feature together with `progress_tracking`. ([#43](https://github.com/NiklasEi/bevy_asset_loader/issues/43))
- Allow adding dynamic asset collection files to a resource instead of only the Plugin builder
- Make file endings for dynamic asset collection files configurable ([#38](https://github.com/NiklasEi/bevy_asset_loader/issues/38))
- Expose `DynamicAssetCollection` to give access to the Bevy `Assets<DynamicAssetCollection>` resource
- Support loading lists of files as `Vec<HandleUntyped>` or `Vec<Handle<T>>` ([#48](https://github.com/NiklasEi/bevy_asset_loader/issues/48))
  - Also supported as dynamic asset
- `folder` attribute is now called `collection`

## v0.10.0
- Bump Bevy to version `0.7`
- Split `render` feature into `2d` (for texture atlas support) and `3d` (standard material support)
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
