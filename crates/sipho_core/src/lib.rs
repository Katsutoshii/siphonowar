/// Core libraries used by Siphonowar.
use bevy::prelude::*;

pub mod aabb;
pub mod camera;
pub mod cursor;
pub mod grid;
pub mod meshes;
pub mod physics;
pub mod raycast;
pub mod stages;
pub mod team;
pub mod window;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        camera::{CameraController, CameraMoveEvent, CameraPlugin, MainCamera},
        cursor::{Cursor, CursorAssets, CursorPlugin},
        grid::{GridSize, GridSpec, RowCol, RowColDistance},
        meshes,
        physics::{
            self, Acceleration, InverseTransform, PhysicsBundle, PhysicsMaterial,
            PhysicsMaterialType, PhysicsPlugin, Velocity,
        },
        raycast::{RaycastCommands, RaycastEvent, RaycastTarget},
        stages::SystemStage,
        team::{Team, TeamMaterials},
        window::{self, ScalableWindow},
        zindex, CorePlugin,
    };
}

use prelude::*;

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PhysicsPlugin, CursorPlugin, CameraPlugin));
    }
}
