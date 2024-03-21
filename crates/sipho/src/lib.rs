use bevy::prelude::*;

pub mod camera;
pub mod config;
pub mod console;
pub mod cursor;
pub mod effects;
pub mod grid;
pub mod inputs;
pub mod objects;
pub mod raycast;
pub mod scene;
pub mod selector;
pub mod waypoint;
pub mod window;

pub mod prelude {
    pub use crate::{
        camera::{CameraController, CameraMoveEvent, MainCamera},
        config::Configs,
        cursor::Cursor,
        effects,
        effects::EffectCommands,
        grid::{
            CreateWaypointEvent, EntityGridEvent, EntitySet, Grid2, Grid2Plugin, GridEntity,
            GridSize, GridSpec, NavigationGrid2, Obstacle, RowCol, RowColDistance,
        },
        inputs::{ControlAction, ControlEvent},
        objects::{
            DamageEvent, Health, InteractionConfigs, Object, ObjectCommands, ObjectConfig,
            ObjectConfigs, Objective, ObjectiveConfig, ObjectiveDebugger, Objectives, Team,
        },
        raycast::{RaycastEvent, RaycastTarget},
        selector::Selected,
        waypoint::Waypoint,
        window, SiphonowarPlugin,
    };
    pub use sipho_core::prelude::*;
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
            console::CustomConsolePlugin,
            scene::LoadableScenePlugin,
            selector::SelectorPlugin,
            waypoint::WaypointPlugin,
            raycast::RaycastPlugin,
            camera::CameraPlugin,
            cursor::CursorPlugin,
            effects::EffectsPlugin,
        ))
        .add_systems(
            FixedUpdate,
            window::resize_window.in_set(SystemStage::Spawn),
        );
    }
}
