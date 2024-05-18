use bevy::app::AppExit;

use crate::prelude::*;
use bevy_bundletree::*;

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
enum PauseMenuButtonAction {
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

#[derive(BundleEnum, IntoBundleTree)]
enum UiNode {
    Node(NodeBundle),
    Text(TextBundle),
    Image(ImageBundle),
    PauseMenu(PauseMenuBundle),
    PauseMenuButton((ButtonBundle, PauseMenuButtonAction)),
}

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

#[derive(Bundle)]
pub struct PauseMenuBundle {
    pub pause_menu: PauseMenu,
    pub name: Name,
    pub node: NodeBundle,
}
impl Default for PauseMenuBundle {
    fn default() -> Self {
        Self {
            pause_menu: PauseMenu,
            name: Name::new("Pause Menu"),
            node: NodeBundle {
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
        }
    }
}

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

        let tree: BundleTree<UiNode> = PauseMenuBundle::default().with_children([
            // Flex column
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            }
            .with_children([
                // Game title
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
                })
                .into_tree(),
                // Resume button
                (
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    PauseMenuButtonAction::Play,
                )
                    .with_children([
                        ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage::new(asset_server.load("textures/icons/right.png")),
                            ..default()
                        }
                        .into_tree(),
                        TextBundle::from_section("Resume", button_text_style.clone()).into_tree(),
                    ]),
                // Settings button
                (
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    PauseMenuButtonAction::Settings,
                )
                    .with_children([
                        ImageBundle {
                            style: button_icon_style.clone(),
                            image: UiImage::new(asset_server.load("textures/icons/wrench.png")),
                            ..default()
                        }
                        .into_tree(),
                        TextBundle::from_section("Settings", button_text_style.clone()).into_tree(),
                    ]),
                // Quit button
                (
                    ButtonBundle {
                        style: button_style,
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    PauseMenuButtonAction::Quit,
                )
                    .with_children([
                        ImageBundle {
                            style: button_icon_style,
                            image: UiImage::new(asset_server.load("textures/icons/exit_right.png")),
                            ..default()
                        }
                        .into_tree(),
                        TextBundle::from_section("Quit", button_text_style).into_tree(),
                    ]),
                // Help text
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
                })
                .into_tree(),
            ]),
        ]);
        commands.spawn_tree(tree);
    }

    // Handle changing all buttons color based on mouse interaction
    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (
                Changed<Interaction>,
                With<Button>,
                With<PauseMenuButtonAction>,
            ),
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
            (&Interaction, &PauseMenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut physics_state: ResMut<NextState<PhysicsSimulationState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    PauseMenuButtonAction::Quit => {
                        app_exit_events.send(AppExit);
                    }
                    PauseMenuButtonAction::Play => {
                        game_state.set(GameState::Running);
                        physics_state.set(PhysicsSimulationState::Running);
                        menu_state.set(MenuState::Disabled);
                    }
                    PauseMenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                }
            }
        }
    }

    fn show(mut query: Query<&mut Visibility, With<PauseMenu>>) {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    }

    fn hide(mut query: Query<&mut Visibility, With<PauseMenu>>) {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
