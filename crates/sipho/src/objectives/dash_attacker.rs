use std::time::Duration;

use crate::prelude::*;

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
// Entity will attack nearest enemy in surrounding grid
pub struct DashAttack {
    pub entity: Entity,
    pub frame: u16,
    pub cooldown: Timer,
}
impl DashAttack {
    /// Gets a random attack delay.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..100))
    }

    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(500..1000))
    }
}
