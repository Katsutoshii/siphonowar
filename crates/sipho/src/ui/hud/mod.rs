use crate::prelude::*;

use self::{
    assets::HudAssets,
    controls_pane::{HudControlsButton, HudControlsButtonBundle, HudControlsPane},
    minimap::{MinimapUi, MinimapUiBundle},
    selected_pane::{HudSelectedPane, HudSelectedPaneBundle, HudUnitButton, HudUnitButtonBundle},
};
use bevy_bundletree::*;

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
                    HudControlsButton::button_system,
                    HudSelectedPane::update,
                    HudUnitButton::update,
                ),
            )
            .add_systems(
                FixedUpdate,
                HudControlsButton::update
                    .in_set(FixedUpdateStage::Control)
                    .before(ControlEvent::update),
            );
    }
}

#[derive(Clone, Bundle)]
pub struct HudRootBundle {
    name: Name,
    node: NodeBundle,
}
impl Default for HudRootBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Hud"),
            node: NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(IntoBundleTree, BundleEnum)]
pub enum HudUiNode {
    Root(HudRootBundle),
    Node(NodeBundle),
    Text(TextBundle),
    ControlsButton(HudControlsButtonBundle),
    UnitButton(HudUnitButtonBundle),
    SelectedPane(HudSelectedPaneBundle),
    Minimap(MinimapUiBundle),
}

pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.35);
pub const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.45, 0.45, 0.45);

fn setup(mut commands: Commands, assets: Res<HudAssets>) {
    commands.spawn_tree(
        // Root
        HudRootBundle::default().with_children([
            // Flex Row
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    display: Display::Flex,
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            }
            .with_children([
                HudControlsPane.tree(&assets),
                HudSelectedPane.tree(&assets),
                MinimapUi.tree(&assets),
            ]),
        ]),
    );
}
