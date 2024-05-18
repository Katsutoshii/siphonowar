use bevy::input::ButtonState;

use super::*;

/// Tag component used to mark which setting is currently selected
#[derive(Component)]
pub struct SelectedOption;

pub struct HudControlsPane;
impl MakeBundleTree<HudUiNode, &HudAssets> for HudControlsPane {
    fn tree(self, assets: &HudAssets) -> BundleTree<HudUiNode> {
        NodeBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(300.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
                ..default()
            },
            background_color: Color::DARK_GRAY.with_a(0.2).into(),
            ..default()
        }
        .with_children({
            let mut children = vec![];
            for button in [
                HudControlsButton {
                    text: "1".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "2".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "3".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "4".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "Q".to_string(),
                    description: "Worker".to_string(),
                    action: Some(ControlAction::BuildWorker),
                },
                HudControlsButton {
                    text: "W".to_string(),
                    description: "Armor".to_string(),
                    action: Some(ControlAction::BuildArmor),
                },
                HudControlsButton {
                    text: "E".to_string(),
                    description: "Shocker".to_string(),
                    action: Some(ControlAction::BuildShocker),
                },
                HudControlsButton {
                    text: "R".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "A".to_string(),
                    description: "Attack".to_string(),
                    action: Some(ControlAction::AttackMode),
                },
                HudControlsButton {
                    text: "S".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "D".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "F".to_string(),
                    description: "".to_string(),
                    action: None,
                },
                HudControlsButton {
                    text: "Z".to_string(),
                    description: "Auto".to_string(),
                    action: Some(ControlAction::SpawnZooid),
                },
                HudControlsButton {
                    text: "X".to_string(),
                    description: "Head".to_string(),
                    action: Some(ControlAction::SpawnHead),
                },
                HudControlsButton {
                    text: "C".to_string(),
                    description: "Connect".to_string(),
                    action: Some(ControlAction::TieSelection),
                },
                HudControlsButton {
                    text: "V".to_string(),
                    description: "Pair".to_string(),
                    action: Some(ControlAction::TieCursor),
                },
            ] {
                children.push(button.tree(assets));
            }
            children
        })
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct HudControlsButton {
    pub text: String,
    pub description: String,
    pub action: Option<ControlAction>,
}
#[derive(Bundle)]
pub struct HudControlsButtonBundle {
    pub data: HudControlsButton,
    pub button: ButtonBundle,
}
impl Default for HudControlsButtonBundle {
    fn default() -> Self {
        Self {
            data: HudControlsButton::default(),
            button: ButtonBundle {
                style: Style {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceEvenly,
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: Color::DARK_GRAY.with_a(0.4).into(),
                ..default()
            },
        }
    }
}
impl MakeBundleTree<HudUiNode, &HudAssets> for HudControlsButton {
    fn tree(self, _assets: &HudAssets) -> BundleTree<HudUiNode> {
        HudControlsButtonBundle {
            data: self.clone(),
            ..default()
        }
        .with_children([
            TextBundle {
                text: Text::from_section(
                    self.text,
                    TextStyle {
                        font_size: 18.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            }
            .into_tree(),
            TextBundle {
                text: Text::from_section(
                    self.description,
                    TextStyle {
                        font_size: 12.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            }
            .into_tree(),
        ])
    }
}

impl HudControlsButton {
    pub fn update(
        mut buttons: Query<(&HudControlsButton, &mut Interaction)>,
        mut events: EventReader<ControlEvent>,
    ) {
        let events: Vec<&ControlEvent> = events.read().collect();
        for (button, mut interaction) in buttons.iter_mut() {
            for event in events.iter() {
                if button.action == Some(event.action) {
                    match event.state {
                        ButtonState::Pressed => *interaction = Interaction::Pressed,
                        ButtonState::Released => *interaction = Interaction::None,
                    }
                }
            }
        }
    }

    // Handle changing all buttons color based on mouse interaction
    pub fn button_system(
        mut interaction_query: Query<
            (
                &HudControlsButton,
                &Interaction,
                &mut BackgroundColor,
                Option<&SelectedOption>,
            ),
            Changed<Interaction>,
        >,
    ) {
        for (_button, interaction, mut color, selected) in &mut interaction_query {
            *color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }
}
