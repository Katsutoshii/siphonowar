/// Objectives define what an object is trying to do.
/// We maintain a stack of objectives for each object.
/// Each frame, we check the current object and try to resolve it to the corresponding behavior components.
use crate::prelude::*;

pub mod config;
pub mod cooldown;
pub mod dash_attacker;
pub mod debug;
pub mod navigator;
pub mod objective;
pub mod shock_attacker;
pub mod stun;

pub use {
    config::ObjectiveConfig,
    cooldown::Cooldown,
    dash_attacker::DashAttacker,
    debug::ObjectiveDebugger,
    navigator::Navigator,
    objective::{Objective, Objectives},
    stun::Stunned,
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
                (
                    Objectives::set_objective,
                    Objectives::update,
                    ObjectiveDebugger::update,
                )
                    .chain()
                    .in_set(FixedUpdateStage::AI)
                    .in_set(GameStateSet::Running),
            )
            .add_plugins((
                navigator::NavigatorPlugin,
                dash_attacker::DashAttackerPlugin,
                shock_attacker::ShockAttackerPlugin,
                cooldown::CooldownPlugin,
                stun::StunPlugin,
            ));
    }
}
