/// Core libraries used by Siphonowar.
use bevy::prelude::*;

pub mod aabb;
pub mod meshes;
pub mod physics;
pub mod raycast;
pub mod rowcol;
pub mod stages;
pub mod team;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        meshes,
        physics::{
            self, Acceleration, PhysicsBundle, PhysicsMaterial, PhysicsMaterialType, PhysicsPlugin,
            Velocity,
        },
        raycast::{RaycastCommands, RaycastEvent, RaycastTarget},
        rowcol::{RowCol, RowColDistance},
        stages::SystemStage,
        team::{Team, TeamMaterials},
        zindex, CorePlugin,
    };
}

use prelude::*;

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugin);
    }
}
