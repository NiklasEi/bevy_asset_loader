use bevy::app::AppExit;
use bevy::asset::AssetPlugin;
use bevy::image::{
    CompressedImageFormats, Image, ImageLoader, ImageLoaderSettings, ImageSampler,
    ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::prelude::*;

#[test]
fn settings_attribute_applies_loader_settings() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin));
    // Register the image asset and its loader so the collection can load a
    // `.png` in a headless test (there is no `RenderApp` here).
    app.init_asset::<Image>();
    app.register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE));

    app.init_state::<MyStates>();
    app.add_loading_state(
        LoadingState::new(MyStates::Load)
            .continue_to_state(MyStates::Next)
            .load_collection::<ImageAssets>(),
    )
    .add_systems(Update, timeout.run_if(in_state(MyStates::Load)))
    .add_systems(OnEnter(MyStates::Next), expect)
    .run();
}

fn timeout(time: Res<Time>) {
    if time.elapsed_secs_f64() > 60. {
        panic!("The asset loader did not change the state in 60 seconds");
    }
}

fn expect(
    collection: Option<Res<ImageAssets>>,
    images: Res<Assets<Image>>,
    mut exit: MessageWriter<AppExit>,
) {
    let collection = collection.expect("The asset collection was not inserted");
    let image = images
        .get(&collection.player)
        .expect("Image should be added to its asset resource");
    let ImageSampler::Descriptor(descriptor) = &image.sampler else {
        panic!("The `settings` closure was not applied: expected a nearest sampler descriptor");
    };
    assert_eq!(
        descriptor.as_wgpu(),
        ImageSamplerDescriptor::nearest().as_wgpu()
    );
    exit.write(AppExit::Success);
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/player.png", settings = |settings: &mut ImageLoaderSettings| {
        settings.sampler = ImageSampler::nearest();
    })]
    player: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    Load,
    Next,
}
