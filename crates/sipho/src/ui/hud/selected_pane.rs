use super::*;
use bevy::color::palettes::css::DARK_GRAY;
use bevy::utils::HashMap;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HudSelectedPane;
impl MakeBundleTree<HudUiNode, &HudAssets> for HudSelectedPane {
    fn tree(self, assets: &HudAssets) -> BundleTree<HudUiNode> {
        HudSelectedPaneBundle::default().with_children({
            let mut children = Vec::new();
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
                children.push(button.tree(assets));
            }
            children
        })
    }
}

#[derive(Bundle)]
pub struct HudSelectedPaneBundle {
    pub data: HudSelectedPane,
    pub node: NodeBundle,
}
impl Default for HudSelectedPaneBundle {
    fn default() -> Self {
        Self {
            data: HudSelectedPane,
            node: NodeBundle {
                style: Style {
                    width: Val::Px(600.0),
                    height: Val::Px(150.0),
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(8, 1.0),
                    grid_template_rows: RepeatedGridTrack::flex(2, 1.0),
                    ..default()
                },
                background_color: DARK_GRAY.with_alpha(0.2).into(),
                ..default()
            },
        }
    }
}

impl HudSelectedPane {
    pub fn update(
        selection: Query<&Object, With<Selected>>,
        ui: Query<(&Self, &Children)>,
        mut buttons: Query<&mut HudUnitButton>,
    ) {
        let mut objects: HashMap<Object, usize> = HashMap::new();
        for object in selection.iter() {
            *objects.entry(*object).or_insert(0) += 1;
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
#[derive(Bundle)]
pub struct HudUnitButtonBundle {
    pub data: HudUnitButton,
    pub button: ButtonBundle,
}
impl Default for HudUnitButtonBundle {
    fn default() -> Self {
        Self {
            data: HudUnitButton::default(),
            button: ButtonBundle {
                style: Style {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    ..default()
                },
                background_color: DARK_GRAY.with_alpha(0.4).into(),
                ..default()
            },
        }
    }
}
impl MakeBundleTree<HudUiNode, &HudAssets> for HudUnitButton {
    fn tree(self, _assets: &HudAssets) -> BundleTree<HudUiNode> {
        HudUnitButtonBundle {
            data: self.clone(),
            ..default()
        }
        .with_children([TextBundle {
            text: Text::from_section(
                self.text,
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
        }
        .into_tree()])
    }
}
impl HudUnitButton {
    pub fn update(buttons: Query<(&Self, &Children)>, mut text: Query<&mut Text>) {
        for (button, children) in buttons.iter() {
            for child in children.iter() {
                if let Ok(mut text) = text.get_mut(*child) {
                    text.sections[0].value.clone_from(&button.text);
                }
            }
        }
    }
}
