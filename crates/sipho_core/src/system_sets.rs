use crate::prelude::*;
use bevy::ecs::schedule::SystemSetConfigs;

pub struct SystemSetPlugin;
impl Plugin for SystemSetPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(FixedUpdate, FixedUpdateStage::get_config())
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
pub enum FixedUpdateStage {
    /// Spawn entities that are not objects.
    /// Can reference transforms from objects spawned in last frame's ObjectSpawn.
    Spawn,
    /// Systems after spawning objects.
    PostSpawn,
    /// Find nearest neighbors.
    FindNeighbors,
    /// Run AI logic based on neighbors.
    AI,
    /// Compute forces between objects
    AccumulateForces,
    /// Apply physics integration.
    Physics,
    /// Operations after applying physics integration.
    PostPhysics,
    /// Process despawn events and clean up references.
    PreDespawn,
    /// Despawn dead or orphaned entities.
    Despawn,
    /// Cleanup
    Cleanup,
}
impl FixedUpdateStage {
    pub fn get_config() -> SystemSetConfigs {
        (
            Self::Spawn,
            Self::PostSpawn,
            Self::FindNeighbors,
            Self::AI,
            Self::AccumulateForces,
            Self::Physics,
            Self::PostPhysics,
            Self::PreDespawn,
            Self::Despawn,
            Self::Cleanup,
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
