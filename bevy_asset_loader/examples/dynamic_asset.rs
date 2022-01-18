use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetKeys, AssetLoader, DynamicAsset};

const PLAYER_SPEED: f32 = 5.;

/// This example shows how to load an asset collection with dynamic assets
fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        // this collection has a dynamic asset where the file path is resolved at run time
        // => see the TextureAssets definition below
        .with_collection::<ImageAssets>()
        .with_collection::<AudioAssets>()
        .build(&mut app);
    AssetLoader::new(MyStates::MenuAssetLoading)
        .continue_to_state(MyStates::Menu)
        .with_collection::<FontAssets>()
        .build(&mut app);
    app.add_state(MyStates::MenuAssetLoading)
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_system_set(
            SystemSet::on_enter(MyStates::Next)
                .with_system(spawn_player_and_tree.system())
                .with_system(play_background_audio.system()),
        )
        .add_system_set(SystemSet::on_enter(MyStates::Menu).with_system(menu.system()))
        .add_system_set(SystemSet::on_update(MyStates::Menu).with_system(choose_character.system()))
        .add_system_set(SystemSet::on_exit(MyStates::Menu).with_system(exit_menu.system()))
        .add_system_set(SystemSet::on_update(MyStates::Next).with_system(move_player.system()))
        .run();
}

#[derive(AssetCollection)]
struct ImageAssets {
    // This key will be resolved when the collection is loaded.
    // It needs to be registered in the resource bevy_asset_loader::AssetKeys
    // => see the choose_character system below
    #[asset(key = "character")]
    player: Handle<Image>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
}

// This system decides which file to load as the character sprite based on some player input
fn choose_character(
    mut state: ResMut<State<MyStates>>,
    mut asset_keys: ResMut<AssetKeys>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        asset_keys.register_asset(
            "character",
            DynamicAsset::File {
                path: "images/female_adventurer.png".to_owned(),
            },
        );
    } else if mouse_input.just_pressed(MouseButton::Right) {
        asset_keys.register_asset(
            "character",
            DynamicAsset::File {
                path: "images/zombie.png".to_owned(),
            },
        );
    } else {
        return;
    }

    state
        .set(MyStates::AssetLoading)
        .expect("Failed to change state");
}

// Rest of example setup

#[derive(AssetCollection)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands
        .spawn_bundle(SpriteBundle {
            texture: image_assets.player.clone(),
            transform,
            ..Default::default()
        })
        .insert(Player);
    commands.spawn_bundle(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(50., 30., 1.)),
        ..Default::default()
    });
}

fn play_background_audio(audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.play(audio_assets.background.clone());
}

fn move_player(input: Res<Input<KeyCode>>, mut player: Query<&mut Transform, With<Player>>) {
    let mut movement = Vec3::new(0., 0., 0.);
    if input.pressed(KeyCode::W) {
        movement.y += 1.;
    }
    if input.pressed(KeyCode::S) {
        movement.y -= 1.;
    }
    if input.pressed(KeyCode::A) {
        movement.x -= 1.;
    }
    if input.pressed(KeyCode::D) {
        movement.x += 1.;
    }
    if movement == Vec3::ZERO {
        return;
    }
    movement = movement.normalize() * PLAYER_SPEED;
    if let Ok(mut transform) = player.get_single_mut() {
        transform.translation += movement;
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    MenuAssetLoading,
    Menu,
    AssetLoading,
    Next,
}

fn menu(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            visibility: Visibility {
                is_visible: false,
            },
            ..Default::default()
        })
        .insert(MenuUi)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value:
                            "Choose your character: Adventurer (left click) or Zombie (right click)\n\
                            Your input will decide which image file gets loaded"
                                .to_string(),
                        style: TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 30.0,
                            color: Color::rgb(1., 1., 1.),
                        },
                    }],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
                ..Default::default()
            });
        });
}

fn exit_menu(mut commands: Commands, ui: Query<Entity, With<MenuUi>>) {
    for entity in ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MenuUi;