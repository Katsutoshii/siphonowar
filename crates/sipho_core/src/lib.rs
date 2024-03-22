/// Core libraries used by Siphonowar.
pub mod aabb;
pub mod camera;
pub mod cursor;
pub mod grid;
pub mod inputs;
pub mod meshes;
pub mod nav;
pub mod physics;
pub mod raycast;
pub mod shader_plane;
pub mod stages;
pub mod team;
pub mod window;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        camera::{CameraController, CameraMoveEvent, MainCamera},
        cursor::{Cursor, CursorAssets},
        grid::{
            EntityGridEvent, EntitySet, Grid2, Grid2Plugin, GridEntity, GridSize, GridSpec,
            Obstacle, RowCol, RowColDistance, SparseGrid2, VisibilityUpdate, VisibilityUpdateEvent,
        },
        inputs::{ControlAction, ControlEvent},
        meshes,
        nav::{CreateWaypointEvent, NavigationCostEvent, NavigationGrid2, SparseFlowGrid2},
        physics::{
            self, Acceleration, PhysicsBundle, PhysicsMaterial, PhysicsMaterialType, Velocity,
        },
        raycast::{RaycastCommands, RaycastEvent, RaycastTarget},
        shader_plane::{ShaderPlaneAssets, ShaderPlaneMaterial, ShaderPlanePlugin},
        stages::SystemStage,
        team::{Team, TeamConfig, TeamMaterials},
        window::{self, ScalableWindow},
        zindex, CorePlugin,
    };
    pub use bevy::prelude::*;
}

use prelude::*;

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            team::TeamPlugin,
            inputs::InputActionPlugin,
            grid::GridPlugin,
            nav::NavigationPlugin,
            physics::PhysicsPlugin,
            cursor::CursorPlugin,
            camera::CameraPlugin,
        ));
    }
}
