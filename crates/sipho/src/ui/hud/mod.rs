use crate::prelude::*;

use self::{
    assets::HudAssets,
    controls_pane::{HudControlsButton, HudControlsButtonBundle, HudControlsPane},
    minimap::{MinimapUi, MinimapUiBundle},
    selected_pane::{HudSelectedPane, HudSelectedPaneBundle, HudUnitButton, HudUnitButtonBundle},
};
use bevy::ecs::system::EntityCommands;
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
                    (HudControlsButton::update, HudControlsButton::button_system).chain(),
                    HudSelectedPane::update,
                    HudUnitButton::update,
                ),
            );
    }
}

#[derive(Bundle, Clone)]
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
#[derive(Default, From)]
pub enum HudUiNode {
    #[default]
    None,
    Root(#[from] HudRootBundle),
    Node(#[from] NodeBundle),
    Text(#[from] TextBundle),
    ControlsButton(#[from] HudControlsButtonBundle),
    UnitButton(#[from] HudUnitButtonBundle),
    SelectedPane(#[from] HudSelectedPaneBundle),
    Minimap(#[from] MinimapUiBundle),
}
impl SpawnableBundle for HudUiNode {
    fn spawn<'w>(self, commands: &'w mut Commands) -> EntityCommands<'w> {
        match self {
            Self::None => commands.spawn(()),
            Self::Root(bundle) => commands.spawn(bundle),
            Self::Node(bundle) => commands.spawn(bundle),
            Self::Text(bundle) => commands.spawn(bundle),
            Self::ControlsButton(bundle) => commands.spawn(bundle),
            Self::UnitButton(bundle) => commands.spawn(bundle),
            Self::SelectedPane(bundle) => commands.spawn(bundle),
            Self::Minimap(bundle) => commands.spawn(bundle),
        }
    }
    fn spawn_child<'w>(self, commands: &'w mut ChildBuilder<'_>) -> EntityCommands<'w> {
        match self {
            Self::None => commands.spawn(()),
            Self::Root(bundle) => commands.spawn(bundle),
            Self::Node(bundle) => commands.spawn(bundle),
            Self::Text(bundle) => commands.spawn(bundle),
            Self::ControlsButton(bundle) => commands.spawn(bundle),
            Self::UnitButton(bundle) => commands.spawn(bundle),
            Self::SelectedPane(bundle) => commands.spawn(bundle),
            Self::Minimap(bundle) => commands.spawn(bundle),
        }
    }
}

pub const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.35, 0.35);
pub const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.45, 0.45, 0.45);

fn setup(mut commands: Commands, assets: Res<HudAssets>) {
    commands.spawn_tree(BundleTree::new(HudRootBundle::default()).with_children(
        vec![BundleTree::new(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    display: Display::Flex,
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            }).with_children(vec![
                HudControlsPane.tree(&assets),
                HudSelectedPane.tree(&assets),
                MinimapUi.tree(&assets),
            ])],
    ));
}
