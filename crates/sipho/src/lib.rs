pub mod objectives;
pub mod objects;
pub mod scene;
pub mod selector;
pub mod waypoint;

pub mod prelude {
    pub use sipho_core::prelude::*;
    pub use sipho_vfx::prelude::*;

    pub use crate::{
        objectives::{Objective, ObjectiveConfig, ObjectiveDebugger, Objectives},
        objects::{
            CarriedBy, CarryEvent, DamageEvent, Health, InteractionConfigs, Object, ObjectCommands,
            ObjectConfig, ObjectConfigs, ObjectSpec,
        },
        selector::Selected,
        waypoint::{Waypoint, WaypointAssets},
        SiphonowarPlugin,
    };
}

use prelude::*;

pub struct SiphonowarPlugin;
impl Plugin for SiphonowarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(window::custom_plugin()),
            CorePlugin,
            objects::ObjectsPlugin,
            objectives::ObjectivePlugin,
            scene::LoadableScenePlugin,
            selector::SelectorPlugin,
            waypoint::WaypointPlugin,
            sipho_vfx::VfxPlugin,
        ));
    }
}
