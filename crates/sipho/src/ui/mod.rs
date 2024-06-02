use crate::prelude::*;

pub mod hud;
pub mod pause_menu;
pub mod selector;
pub mod waypoint;

pub use {
    selector::{Selectable, Selected},
    waypoint::Waypoint,
};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            hud::HudPlugin,
            pause_menu::PauseMenuPlugin,
            selector::SelectorPlugin,
            waypoint::WaypointPlugin,
        ));
    }
}
