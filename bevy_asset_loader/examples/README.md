# Examples

These examples are simple Bevy Apps illustrating the capabilities of `bevy_asset_loader`. Run the examples
with `cargo run --example <example>`.

| Example                                                    | Description                                                              |
|------------------------------------------------------------|--------------------------------------------------------------------------|
| [`atlas_from_grid.rs`](atlas_from_grid.rs)                 | Loading a texture atlas from a sprite sheet                              |
| [`custom_dynamic_assets.rs`](custom_dynamic_assets.rs)     | Define and use your own dynamic assets                                   |
| [`dynamic_asset.rs`](dynamic_asset.rs)                     | Load dynamic assets from a `.ron` file                                   |
| [`failure_state.rs`](failure_state.rs)                     | Sets up a failure state                                                  |
| [`full_collection.rs`](full_collection.rs)                 | A complete asset collection with all supported non-dynamic field types   |
| [`full_dynamic_collection.rs`](full_dynamic_collection.rs) | A complete asset collection with all supported dynamic asset field types |
| [`image_asset.rs`](image_asset.rs)                         | How to set different samplers for image assets                           |
| [`init_resource.rs`](init_resource.rs)                     | Inserting a `FromWorld` resource when all asset collections are loaded   |
| [`manual_dynamic_asset.rs`](manual_dynamic_asset.rs)       | Load an image asset from a path resolved at run time                     |
| [`no_loading_state.rs`](no_loading_state.rs)               | How to use asset collections without a loading state                     |
| [`progress_tracking.rs`](progress_tracking.rs)             | How to set up progress tracking using `iyes_progress`                    |
| [`standard_material.rs`](standard_material.rs)             | Loading a standard material from a png file                              |
| [`two_collections.rs`](two_collections.rs)                 | Load multiple asset collections                                          |
| [`asset_maps.rs`](asset_maps.rs)                           | Shows how to use different types as keys in asset maps                   |

## Credits

The examples include third party assets:

Background audio: [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/)
by [Jay_You](https://freesound.org/people/Jay_You/sounds/460432/)

Toon character sheets [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/)
by [Kenny](https://kenney.nl/assets/toon-characters-1)

Pixelart tree [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/)
by [Kenny](https://www.kenney.nl/assets/tiny-town)
