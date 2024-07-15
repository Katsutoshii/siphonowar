use bevy::color::palettes::css::DARK_GRAY;
use bevy::{
    input::ButtonState,
    utils::{HashMap, HashSet},
};
use std::time::Duration;

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
            background_color: DARK_GRAY.with_alpha(0.2).into(),
            ..default()
        }
        .with_children(
            [
                InputAction::Control1,
                InputAction::Control2,
                InputAction::Control3,
                InputAction::Control4,
                InputAction::Grid11,
                InputAction::Grid12,
                InputAction::Grid13,
                InputAction::Grid14,
                InputAction::Grid21,
                InputAction::Grid22,
                InputAction::Grid23,
                InputAction::Grid24,
                InputAction::Grid31,
                InputAction::Grid32,
                InputAction::Grid33,
                InputAction::Grid34,
            ]
            .iter()
            .copied()
            .map(|action| {
                let text = match action {
                    InputAction::Control1 => "1",
                    InputAction::Control2 => "2",
                    InputAction::Control3 => "3",
                    InputAction::Control4 => "4",
                    InputAction::Grid11 => "Q",
                    InputAction::Grid12 => "W",
                    InputAction::Grid13 => "E",
                    InputAction::Grid14 => "R",
                    InputAction::Grid21 => "A",
                    InputAction::Grid22 => "S",
                    InputAction::Grid23 => "D",
                    InputAction::Grid24 => "F",
                    InputAction::Grid31 => "Z",
                    InputAction::Grid32 => "X",
                    InputAction::Grid33 => "C",
                    InputAction::Grid34 => "V",
                    _ => "",
                }
                .to_string();
                HudControlsButton {
                    text,
                    action,
                    control: None,
                }
                .tree(assets)
            }),
        )
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct HudControlsButton {
    pub text: String,
    pub action: InputAction,
    pub control: Option<ControlAction>,
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
                    justify_content: JustifyContent::Default,
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: DARK_GRAY.with_alpha(0.4).into(),
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
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    justify_self: JustifySelf::Start,
                    align_self: AlignSelf::Start,
                    margin: UiRect::all(Val::Px(10.0)),
                    height: Val::Px(20.),
                    ..default()
                },
                ..default()
            }
            .into_tree(),
            TextBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font_size: 11.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    justify_self: JustifySelf::End,
                    align_self: AlignSelf::End,
                    margin: UiRect::all(Val::Px(10.0)),
                    height: Val::Px(20.),
                    ..default()
                },
                ..default()
            }
            .into_tree(),
        ])
    }
}

impl HudControlsButton {
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut buttons: Query<(Entity, &mut HudControlsButton, &mut Interaction, &Children)>,
        mut button_text: Query<&mut Text>,
        mut inputs: EventReader<InputEvent>,
        selected: Query<&Object, (With<Selected>, Without<HudControlsButton>)>,
        configs: Res<ObjectConfigs>,
        mut controls: EventWriter<ControlEvent>,
        mut raycasts: EventReader<RaycastEvent>,
        mut state: ResMut<ControlState>,
    ) {
        let objects: HashSet<Object> = selected.iter().copied().collect();

        let mut map: HashMap<InputAction, ControlAction> = HashMap::with_capacity(4);
        map.insert(InputAction::Grid24, ControlAction::Plankton);
        map.insert(InputAction::Grid32, ControlAction::Head);
        map.insert(InputAction::Grid33, ControlAction::TieAll);
        map.insert(InputAction::Grid34, ControlAction::Tie);

        for object in objects.iter() {
            let config = &configs[object];
            for (&input, &control) in config.controls.iter() {
                map.entry(input).or_insert(control);
            }
        }

        let mut action_to_button = HashMap::new();
        for (entity, mut button, _interaction, children) in buttons.iter_mut() {
            let control = map.get(&button.action).copied();
            if button.control != control {
                button.control = control;
                let text = &mut button_text.get_mut(children[1]).unwrap().sections[0].value;
                if let Some(control) = map.get(&button.action) {
                    *text = format!("{:?}", control);
                } else {
                    text.clear();
                }
            }
            action_to_button.insert(button.action, entity);
        }

        if let Some(raycast) = raycasts.read().next() {
            for input in inputs.read() {
                if let Some(&entity) = action_to_button.get(&input.action) {
                    if let Some(&action) = map.get(&input.action) {
                        if input.state == ButtonState::Pressed {
                            state.press_action(action, raycast.target);
                        } else if input.state == ButtonState::Released
                            && !state.press_durations.contains_key(&action)
                        {
                            continue;
                        }

                        // Only update state if no inputs were held last frame.
                        if state.held_actions.is_empty() {
                            if let ControlAction::Attack = action {
                                state.mode = ControlMode::Attack;
                            }
                        }
                        controls.send(ControlEvent {
                            action,
                            state: input.state,
                            position: raycast.world_position,
                            entity: raycast.entity,
                            duration: Duration::default(),
                        });

                        if input.state == ButtonState::Released {
                            state.release_action(action);
                        }
                    }

                    if let Ok((_entity, button, mut interaction, _children)) =
                        buttons.get_mut(entity)
                    {
                        if button.action == input.action {
                            match input.state {
                                ButtonState::Pressed => *interaction = Interaction::Pressed,
                                ButtonState::Released => *interaction = Interaction::None,
                            }
                        }
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
