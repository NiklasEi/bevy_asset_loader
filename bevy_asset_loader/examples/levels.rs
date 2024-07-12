use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(
            Update,
            listen_for_menu_buttons.run_if(in_state(AppState::Menu)),
        )
        .enable_state_scoped_entities::<AppState>()
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    Loading,
    InGame,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::InGame)]
enum Level {
    #[default]
    Forest,
    Desert,
}

#[derive(Component)]
struct PrepareLevel(Level);

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(15.),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            StateScoped(AppState::Menu),
        ))
        .with_children(|parent| {
            parent
                .spawn((menu_button(), PrepareLevel(Level::Forest)))
                .with_children(|children| {
                    children.spawn(TextBundle::from_section("Forest", Default::default()));
                });
            parent
                .spawn((menu_button(), PrepareLevel(Level::Desert)))
                .with_children(|children| {
                    children.spawn(TextBundle::from_section("Desert", Default::default()));
                });
        });
}

fn menu_button() -> impl Bundle {
    ButtonBundle {
        background_color: BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
        style: Style {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Px(200.),
            height: Val::Px(40.),
            ..default()
        },
        ..default()
    }
}

fn listen_for_menu_buttons(
    clicks: Query<(&Interaction, &PrepareLevel)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, _prepare_level) in &clicks {
        if matches!(interaction, Interaction::Pressed) {
            next_state.set(AppState::Loading);
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
