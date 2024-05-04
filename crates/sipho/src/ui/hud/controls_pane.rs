use super::*;

// Tag component used to mark which setting is currently selected
#[derive(Component)]
pub struct SelectedOption;

pub struct HudControlsPane;
impl SpawnHudNode for HudControlsPane {
    fn spawn(&self, parent: &mut ChildBuilder, assets: &HudAssets) {
        parent
            .spawn((NodeBundle {
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
            },))
            .with_children(|parent| {
                for button in [
                    HudControlsButton {
                        text: "1".to_string(),
                    },
                    HudControlsButton {
                        text: "2".to_string(),
                    },
                    HudControlsButton {
                        text: "3".to_string(),
                    },
                    HudControlsButton {
                        text: "4".to_string(),
                    },
                    HudControlsButton {
                        text: "Q".to_string(),
                    },
                    HudControlsButton {
                        text: "W".to_string(),
                    },
                    HudControlsButton {
                        text: "E".to_string(),
                    },
                    HudControlsButton {
                        text: "R".to_string(),
                    },
                    HudControlsButton {
                        text: "A".to_string(),
                    },
                    HudControlsButton {
                        text: "S".to_string(),
                    },
                    HudControlsButton {
                        text: "D".to_string(),
                    },
                    HudControlsButton {
                        text: "F".to_string(),
                    },
                    HudControlsButton {
                        text: "Z".to_string(),
                    },
                    HudControlsButton {
                        text: "X".to_string(),
                    },
                    HudControlsButton {
                        text: "C".to_string(),
                    },
                    HudControlsButton {
                        text: "V".to_string(),
                    },
                ] {
                    button.spawn(parent, assets);
                }
            });
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HudControlsButton {
    pub text: String,
}
impl SpawnHudNode for HudControlsButton {
    fn spawn(&self, parent: &mut ChildBuilder, _assets: &HudAssets) {
        parent
            .spawn((
                HudControlsButton {
                    text: self.text.clone(),
                },
                ButtonBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(60.0),
                        justify_self: JustifySelf::Center,
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    background_color: Color::DARK_GRAY.with_a(0.4).into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        &self.text,
                        TextStyle {
                            font_size: 24.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    style: Style {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                });
            });
    }
}
impl HudControlsButton {
    // Handle changing all buttons color based on mouse interaction
    pub fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>, With<HudControlsButton>),
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
}
