use crate::prelude::*;

use self::{
    assets::HudAssets,
    controls_pane::{HudControlsButton, HudControlsPane},
    minimap::MinimapUi,
    selected_pane::{HudSelectedPane, HudUnitButton},
};

pub mod assets;
pub mod controls_pane;
pub mod minimap;
pub mod selected_pane;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((minimap::MinimapPlugin,))
            .init_resource::<HudAssets>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    (HudControlsButton::update, HudControlsButton::button_system).chain(),
                    HudSelectedPane::update,
                    HudUnitButton::update,
                ),
            );
    }
}

pub trait SpawnHudNode {
    fn spawn(&self, parent: &mut ChildBuilder, assets: &HudAssets);
}

pub const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.35, 0.35);
pub const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.45, 0.45, 0.45);

pub struct HudFlexRow;
impl SpawnHudNode for HudFlexRow {
    fn spawn(&self, parent: &mut ChildBuilder, assets: &HudAssets) {
        parent
            .spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    display: Display::Flex,
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                HudControlsPane.spawn(parent, assets);
                HudSelectedPane.spawn(parent, assets);
                MinimapUi.spawn(parent, assets);
            });
    }
}

fn setup(mut commands: Commands, assets: Res<HudAssets>) {
    commands
        .spawn((
            Name::new("Hud"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| HudFlexRow.spawn(parent, &assets));
}
