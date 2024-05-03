use crate::prelude::*;

use self::{
    assets::HudAssets, controls_pane::HudControlsPane, minimap::MinimapUi,
    selected_pane::HudSelectedPane,
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
            .add_systems(Startup, setup);
    }
}

pub trait SpawnHudNode {
    fn spawn(parent: &mut ChildBuilder, assets: &HudAssets);
}

pub struct HudFlexRow;
impl SpawnHudNode for HudFlexRow {
    fn spawn(parent: &mut ChildBuilder, assets: &HudAssets) {
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
                HudControlsPane::spawn(parent, assets);
                HudSelectedPane::spawn(parent, assets);
                MinimapUi::spawn(parent, assets);
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
        .with_children(|parent| HudFlexRow::spawn(parent, &assets));
}
