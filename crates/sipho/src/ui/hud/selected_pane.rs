use bevy::utils::HashMap;

use crate::prelude::*;

use super::{assets::HudAssets, SpawnHudNode};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HudSelectedPane;
impl SpawnHudNode for HudSelectedPane {
    fn spawn(&self, parent: &mut ChildBuilder, assets: &HudAssets) {
        parent
            .spawn((
                HudSelectedPane,
                NodeBundle {
                    style: Style {
                        width: Val::Px(600.0),
                        height: Val::Px(150.0),
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(8, 1.0),
                        grid_template_rows: RepeatedGridTrack::flex(2, 1.0),
                        ..default()
                    },
                    background_color: Color::DARK_GRAY.with_a(0.2).into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                for button in [
                    // Row 1
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    // Row 2
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton::default(),
                    HudUnitButton {
                        text: "...".to_string(),
                    },
                ] {
                    button.spawn(parent, assets);
                }
            });
    }
}
impl HudSelectedPane {
    pub fn update(
        selection: Query<(&Object, &Selected)>,
        ui: Query<(&Self, &Children)>,
        mut buttons: Query<&mut HudUnitButton>,
    ) {
        let mut objects: HashMap<Object, usize> = HashMap::new();
        for (object, selected) in selection.iter() {
            if selected.is_selected() {
                *objects.entry(*object).or_insert(0) += 1;
            }
        }
        let mut sorted: Vec<(Object, usize)> = objects.iter().map(|(&k, &v)| (k, v)).collect();
        sorted.sort_by_key(|&(object, _)| object);

        let (_ui, button_ids) = ui.single();
        for (i, button_id) in button_ids.iter().enumerate() {
            if let Ok(mut button) = buttons.get_mut(*button_id) {
                if i < sorted.len() {
                    let (object, count) = sorted[i];
                    button.text = format!("{object:?}\n{count}");
                } else {
                    button.text = "".to_string();
                }
            }
        }
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct HudUnitButton {
    pub text: String,
}
impl SpawnHudNode for HudUnitButton {
    fn spawn(&self, parent: &mut ChildBuilder, _assets: &HudAssets) {
        parent
            .spawn((
                self.clone(),
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
                            font_size: 15.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    style: Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    ..default()
                });
            });
    }
}
impl HudUnitButton {
    pub fn update(buttons: Query<(&Self, &Children)>, mut text: Query<&mut Text>) {
        for (button, children) in buttons.iter() {
            for child in children.iter() {
                if let Ok(mut text) = text.get_mut(*child) {
                    text.sections[0].value = button.text.clone();
                }
            }
        }
    }
}
