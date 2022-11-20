use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::*;

const PLAYER_SPEED: f32 = 5.;

/// This example shows how to load an asset collection with dynamic assets
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_loopless_state(MyStates::MenuAssetLoading)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                // This collection has a dynamic asset where the file path is resolved at run time
                // and one optional image asset for the background.
                // => see the ImageAssets definition below
                .with_collection::<ImageAssets>()
                .with_collection::<AudioAssets>(),
        )
        .add_loading_state(
            LoadingState::new(MyStates::MenuAssetLoading)
                .continue_to_state(MyStates::Menu)
                .with_collection::<FontAssets>(),
        )
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ShowBackground(false))
        .add_enter_system(MyStates::Next, spawn_player_and_tree)
        .add_enter_system(MyStates::Next, play_background_audio)
        .add_enter_system(MyStates::Menu, menu)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(MyStates::Menu)
                .with_system(character_setup)
                .with_system(update_menu)
                .into(),
        )
        .add_exit_system(MyStates::Menu, exit_menu)
        .add_enter_system(MyStates::Next, render_optional_background)
        .add_system(move_player.run_in_state(MyStates::Next))
        .run();
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    // This key will be resolved when the collection is loaded.
    // It needs to be registered in the resource bevy_asset_loader::DynamicAssets
    // => see the choose_character system below
    #[asset(key = "character")]
    player: Handle<Image>,
    // This optional image is only loaded if background is toggled 'On'
    #[asset(key = "background", optional)]
    background: Option<Handle<Image>>,
    #[asset(path = "images/tree.png")]
    tree: Handle<Image>,
}

// This system decides which file to load as the character sprite based on some player input
fn character_setup(
    mut commands: Commands,
    mut dynamic_assets: ResMut<DynamicAssets>,
    mut show_background: ResMut<ShowBackground>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        // Manually register the asset
        //
        // This can be either done with the resource directly:
        dynamic_assets.register_asset(
            "character",
            Box::new(StandardDynamicAsset::File {
                path: "images/female_adventurer.png".to_owned(),
            }),
        );
    } else if mouse_input.just_pressed(MouseButton::Right) {
        // Or be using a command:
        commands.add(RegisterStandardDynamicAsset {
            key: "character",
            asset: StandardDynamicAsset::File {
                path: "images/zombie.png".to_owned(),
            },
        });
    } else if keyboard_input.just_pressed(KeyCode::B) {
        show_background.0 = !show_background.0;
        return;
    } else {
        return;
    }

    if show_background.0 {
        dynamic_assets.register_asset(
            "background",
            Box::new(StandardDynamicAsset::File {
                path: "images/background.png".to_owned(),
            }),
        );
    }
    commands.insert_resource(NextState(MyStates::AssetLoading))
}

#[derive(Resource)]
struct ShowBackground(bool);

// Rest of example setup

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    background: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

fn spawn_player_and_tree(mut commands: Commands, image_assets: Res<ImageAssets>) {
    let mut transform = Transform::from_translation(Vec3::new(0., 0., 1.));
    transform.scale = Vec3::splat(0.5);
    commands
        .spawn(SpriteBundle {
            texture: image_assets.player.clone(),
            transform,
            ..Default::default()
        })
        .insert(Player);
    commands.spawn(SpriteBundle {
        texture: image_assets.tree.clone(),
        transform: Transform::from_translation(Vec3::new(50., 30., 1.)),
        ..Default::default()
    });
}

fn render_optional_background(mut commands: Commands, image_assets: Res<ImageAssets>) {
    if let Some(background) = &image_assets.background {
        commands.spawn(SpriteBundle {
            texture: background.clone(),
            ..Default::default()
        });
    }
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
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgba(0.0,0.0,0.0,1.0)),
            ..Default::default()
        })
        .insert(MenuUi)
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value:
                            "Choose your character: Adventurer (left click) or Zombie (right click)\n\
                            Your input will decide which image file gets loaded\n\n\
                            Press 'B' to toggle background\n\
                            The background image is optional and not loaded if not needed"
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
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Background currently Off".to_string(),
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
            }).insert(BackgroundText);
        });
}

fn update_menu(
    mut background_text: Query<&mut Text, With<BackgroundText>>,
    show_background: Res<ShowBackground>,
) {
    if show_background.is_changed() {
        background_text
            .single_mut()
            .sections
            .get_mut(0)
            .unwrap()
            .value = format!(
            "Background is {}",
            if show_background.0 { "On" } else { "Off" }
        )
    }
}

fn exit_menu(mut commands: Commands, ui: Query<Entity, With<MenuUi>>) {
    for entity in ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct BackgroundText;

#[derive(Component)]
struct MenuUi;
