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

/// Stage of computation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SystemStage {
    Input,
    Spawn,
    PreCompute,
    FindNeighbors,
    Compute,
    PostCompute,
    Apply,
    PostApply,
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
            Self::Despawn,
        )
            .chain()
    }
}

#[derive(SystemSet, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameStateSet {
    #[default]
    Running,
    Paused,
}
