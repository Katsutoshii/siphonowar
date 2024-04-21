/// Core libraries used by Siphonowar.
pub mod aabb;
pub mod camera;
pub mod cursor;
pub mod despawn;
pub mod error;
pub mod game_state;
pub mod grid;
pub mod inputs;
pub mod meshes;
pub mod nav;
pub mod physics;
pub mod pool;
pub mod raycast;
pub mod shader_plane;
pub mod smallset;
pub mod system_sets;
pub mod team;
pub mod window;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        camera::{CameraController, CameraMoveEvent, MainCamera},
        cursor::{Cursor, CursorAssets, CursorParam},
        despawn::{DespawnEvent, ScheduleDespawn},
        error::Error,
        game_state::{AssetLoadState, DebugState, GameState},
        grid::{
            EntityGridEvent, EntitySet, Grid2, Grid2Plugin, GridEntity, GridSize, GridSpec,
            Obstacle, RowCol, RowColDistance, SparseGrid2, TeamEntitySets, VisibilityUpdate,
            VisibilityUpdateEvent,
        },
        inputs::{ControlAction, ControlEvent, ControlMode, ControlState},
        meshes,
        nav::{NavigationCostEvent, NavigationGrid2, SparseFlowGrid2},
        physics::{
            self, Acceleration, PhysicsBundle, PhysicsMaterial, PhysicsMaterialType, Position,
            Velocity,
        },
        pool::EntityPool,
        raycast::{RaycastCommands, RaycastEvent, RaycastTarget},
        shader_plane::{ShaderPlaneAssets, ShaderPlaneMaterial, ShaderPlanePlugin},
        smallset::SmallSet,
        system_sets::{GameStateSet, SystemStage},
        team::{Team, TeamConfig, TeamMaterials, TEAM_BLUE, TEAM_NONE, TEAM_RED},
        window::{self, ScalableWindow},
        zindex, CorePlugin,
    };
    pub use arrayvec::ArrayVec;
    pub use bevy::prelude::*;
}

use prelude::*;

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            despawn::DespawnPlugin,
            game_state::GameStatePlugin,
            system_sets::SystemSetPlugin,
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
