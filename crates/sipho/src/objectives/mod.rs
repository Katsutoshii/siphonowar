use crate::prelude::*;

pub mod dash_attacker;
pub mod debug;
pub mod objective;

pub use {
    debug::ObjectiveDebugger,
    objective::{Objective, Objectives},
};

pub struct ObjectivePlugin;
impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ObjectiveConfig>()
            .register_type::<Objectives>()
            .register_type::<Vec<Objective>>()
            .register_type::<Objective>()
            .add_systems(
                FixedUpdate,
                ((
                    Objectives::update_waypoints,
                    Objectives::update,
                    ObjectiveDebugger::update,
                )
                    .chain()
                    .in_set(SystemStage::PostApply),),
            );
    }
}
