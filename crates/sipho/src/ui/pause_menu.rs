use bevy::app::AppExit;

use crate::prelude::*;

pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, PauseMenu::setup)
            .init_state::<MenuState>()
            .add_systems(
                OnEnter(GameState::Paused),
                (PauseMenu::show, MenuState::set_paused),
            )
            .add_systems(
                OnExit(GameState::Paused),
                (PauseMenu::hide, MenuState::set_disabled),
            )
            .add_systems(
                Update,
                (PauseMenu::button_system, PauseMenu::menu_action)
                    .run_if(in_state(GameState::Paused)),
            );
    }
}

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.35, 0.35);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.45, 0.45, 0.45);

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    Quit,
}

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Disabled,
    Paused,
    Settings,
}
impl MenuState {
    pub fn set_paused(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Paused);
    }
    pub fn set_disabled(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Disabled);
    }
}

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
pub struct PauseMenu;
impl PauseMenu {
    fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        // Common style for all buttons on the screen
        let button_style = Style {
            width: Val::Px(360.0),
            height: Val::Px(64.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_icon_style = Style {
            width: Val::Px(30.0),
            // This takes the icons out of the flexbox flow, to be positioned exactly
            position_type: PositionType::Absolute,
            // The icon will be close to the left border of the button
            left: Val::Px(10.0),
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 24.0,
            color: TEXT_COLOR,
            ..default()
        };

        commands
            .spawn((
                PauseMenu,
                Name::new("Pause Menu"),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    z_index: ZIndex::Global(10),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Display the game name
                        parent.spawn(
                            TextBundle::from_section(
                                "Siphonowar",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 36.0,
                                    color: TEXT_COLOR,
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            }),
                        );

                        // Display three buttons for each action available from the main menu:
                        // - resume
                        // - settings
                        // - quit
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Play,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/icons/right.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style.clone(),
                                    image: UiImage::new(icon),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section(
                                    "Resume",
                                    button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Settings,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/icons/wrench.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style.clone(),
                                    image: UiImage::new(icon),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section(
                                    "Settings",
                                    button_text_style.clone(),
                                ));
                            });
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style,
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Quit,
                            ))
                            .with_children(|parent| {
                                let icon = asset_server.load("textures/icons/exit_right.png");
                                parent.spawn(ImageBundle {
                                    style: button_icon_style,
                                    image: UiImage::new(icon),
                                    ..default()
                                });
                                parent.spawn(TextBundle::from_section("Quit", button_text_style));
                            });

                        parent.spawn(
                            TextBundle::from_section(
                                [
                                    "Create your spawner: 'x'",
                                    "Move camera: move mouse to border",
                                    "Move waypoint: right click",
                                    "Spawn zooids: 'z'",
                                    "Despawn zooids: 'd'",
                                    "Save scene: 's'",
                                    "Open editor: 'e'",
                                    "Spawn food: 'f'",
                                ]
                                .join("\n"),
                                TextStyle {
                                    font_size: 18.0,
                                    ..default()
                                },
                            )
                            .with_style(Style {
                                align_self: AlignSelf::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            }),
                        );
                    });
            });
    }

    // Handle changing all buttons color based on mouse interaction
    #[allow(clippy::type_complexity)]
    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color, selected) in &mut interaction_query {
            *color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    /// Handle applying changes to game state.
    #[allow(clippy::type_complexity)]
    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Quit => {
                        app_exit_events.send(AppExit);
                    }
                    MenuButtonAction::Play => {
                        game_state.set(GameState::Running);
                        menu_state.set(MenuState::Disabled);
                    }
                    MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                }
            }
        }
    }

    fn show(mut query: Query<&mut Visibility, With<PauseMenu>>) {
        for mut visibility in query.iter_mut() {
            info!("Show menu!");
            *visibility = Visibility::Visible;
        }
    }

    fn hide(mut query: Query<&mut Visibility, With<PauseMenu>>) {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
