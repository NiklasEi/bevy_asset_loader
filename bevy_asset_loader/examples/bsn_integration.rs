//! This example shows how to use handles from an [`AssetCollection`] in scenes defined with Bevy's
//! scene notation (bsn).
//!
//! Scenes turn into components through templates, and templates are built with access to the
//! world. The scene functions below take no parameters and contain no asset paths. When the
//! scenes are spawned, the handles are read from the `MyAssets` resource.

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), spawn_scenes)
        .run();
}

/// `Component::from_collection(Collection::field)` builds the component from the collection field.
/// It works for every component that can be created from the field through `From`, which covers
/// `TextFont`, `Sprite`, `Mesh3d`, `MeshMaterial3d`, `AudioPlayer` and friends.
fn text() -> impl Scene {
    bsn! {
        Text2d::from("Built with `TextFont::from_collection`")
        TextFont::from_collection(MyAssets::fira_sans)
        TextColor({Color::srgb(0.4, 0.9, 0.6)})
        Transform::from_xyz(0., 60., 0.)
    }
}

/// The entry builds the whole component, so it cannot be combined with a `TextFont { .. }` patch.
/// Chain `map` to set other fields.
fn sized_text() -> impl Scene {
    bsn! {
        Text2d("...`.map()` to also set the font size")
        TextFont::from_collection(MyAssets::fira_sans)
            .map(|font| font.with_font_size(FontSize::Px(40.)))
        TextColor({Color::srgb(0.6, 0.8, 1.0)})
        Transform::from_xyz(0., 0., 0.)
    }
}

/// Use `from_collection` when the component cannot be created from the handle through
/// a `From` implementation, or when it needs several fields of the collection.
fn closure_text() -> impl Scene {
    bsn! {
        Text2d("...or build it in a `from_collection` closure")
        from_collection(|assets: &MyAssets| {
            TextFont::from_font_size(FontSize::Px(25.)).with_font(assets.fira_sans.clone())
        })
        TextColor({Color::srgb(0.9, 0.8, 0.4)})
        Transform::from_xyz(0., -60., 0.)
    }
}

/// The same works for any other component that can be built from a handle.
fn player() -> impl Scene {
    bsn! {
        Sprite::from_collection(MyAssets::player)
            .map(|sprite| Sprite { custom_size: Some(Vec2::splat(128.)), ..sprite })
        Transform::from_xyz(0., -160., 0.)
    }
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    fira_sans: Handle<Font>,
    #[asset(path = "images/player.png")]
    player: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

fn spawn_scenes(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(text());
    commands.spawn_scene(sized_text());
    commands.spawn_scene(closure_text());
    commands.spawn_scene(player());
}
