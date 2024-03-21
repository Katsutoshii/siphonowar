use bevy::prelude::*;

pub mod config;
pub mod grid;
pub mod inputs;
pub mod objects;
pub mod scene;
pub mod selector;
pub mod waypoint;

pub mod prelude {
    pub use sipho_core::prelude::*;
    pub use sipho_vfx::prelude::*;

    pub use crate::{
        config::Configs,
        grid::{
            CreateWaypointEvent, EntityGridEvent, EntitySet, Grid2, Grid2Plugin, GridEntity,
            NavigationGrid2, Obstacle,
        },
        inputs::{ControlAction, ControlEvent},
        objects::{
            DamageEvent, Health, InteractionConfigs, Object, ObjectCommands, ObjectConfig,
            ObjectConfigs, ObjectSpec, Objective, ObjectiveConfig, ObjectiveDebugger, Objectives,
        },
        selector::Selected,
        waypoint::Waypoint,
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
            config::ConfigPlugin,
            CorePlugin,
            inputs::InputActionPlugin,
            grid::GridPlugin,
            objects::ObjectsPlugin,
            scene::LoadableScenePlugin,
            selector::SelectorPlugin,
            waypoint::WaypointPlugin,
            sipho_vfx::VfxPlugin,
        ));
    }
}
