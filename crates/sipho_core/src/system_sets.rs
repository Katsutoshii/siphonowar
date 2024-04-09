use crate::prelude::*;
use bevy::ecs::schedule::SystemSetConfigs;

pub struct SystemSetPlugin;
impl Plugin for SystemSetPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, SystemStage::get_config())
            .configure_sets(Update, SystemStage::get_config())
            .configure_sets(
                FixedUpdate,
                GameStateSet::Running.run_if(in_state(GameState::Running)),
            )
            .configure_sets(
                Update,
                GameStateSet::Running.run_if(in_state(GameState::Running)),
            );
    }
}

/// Stage of computation for a given frame.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SystemStage {
    /// Process input from the user.
    Input,
    /// Spawn entities for this frame.
    Spawn,
    /// Operations before computation of forces.
    PreCompute,
    /// Find nearest neighbors.
    FindNeighbors,
    /// Compute forces between objects
    Compute,
    /// Operations after computing forces
    PostCompute,
    /// Apply physics integration.
    Apply,
    /// Operations after applying physics integration.
    PostApply,
    /// Compute which entities died and send events.
    Death,
    /// Process death events and clean up references.
    Cleanup,
    /// Despawn dead or orphaned entities.
    Despawn,
}
impl SystemStage {
    pub fn get_config() -> SystemSetConfigs {
        (
            Self::Input,
            Self::Spawn,
            Self::PreCompute,
            Self::FindNeighbors,
            Self::Compute,
            Self::PostCompute,
            Self::Apply,
            Self::PostApply,
            Self::Death,
            Self::Cleanup,
            Self::Despawn,
        )
            .chain()
    }
}

#[derive(SystemSet, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameStateSet {
    /// Game is running.
    #[default]
    Running,
    /// Game is paused.
    Paused,
}
