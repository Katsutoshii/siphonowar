use crate::prelude::*;

use super::{assets::HudAssets, SpawnHudNode};

pub struct HudSelectedPane;
impl SpawnHudNode for HudSelectedPane {
    fn spawn(&self, parent: &mut ChildBuilder, assets: &HudAssets) {
        parent
            .spawn((NodeBundle {
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
            },))
            .with_children(|parent| {
                for button in [
                    HudUnitButton {
                        text: "Head".to_string(),
                    },
                    HudUnitButton {
                        text: "Zooid".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "Test".to_string(),
                    },
                    HudUnitButton {
                        text: "...".to_string(),
                    },
                ] {
                    button.spawn(parent, assets);
                }
            });
    }
}

pub struct HudUnitButton {
    pub text: String,
}
impl SpawnHudNode for HudUnitButton {
    fn spawn(&self, parent: &mut ChildBuilder, _assets: &HudAssets) {
        parent
            .spawn((ButtonBundle {
                style: Style {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    ..default()
                },
                background_color: Color::DARK_GRAY.with_a(0.4).into(),
                ..default()
            },))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        &self.text,
                        TextStyle {
                            font_size: 18.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    style: Style {
                        margin: UiRect::all(Val::Px(10.0)),
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    ..default()
                });
            });
    }
}
