use std::time::Duration;

use crate::prelude::*;

pub struct CooldownPlugin;
impl Plugin for CooldownPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Cooldown>().add_systems(
            FixedUpdate,
            Cooldown::update
                .in_set(FixedUpdateStage::AI)
                .in_set(GameStateSet::Running),
        );
    }
}

/// Cooldown tag component.
#[derive(Component, Reflect, Debug, Deref, DerefMut, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct Cooldown(pub Timer);
impl Cooldown {
    /// Remove completed cooldowns.
    fn update(mut query: Query<(Entity, &mut Cooldown)>, time: Res<Time>, mut commands: Commands) {
        for (entity, mut cooldown) in query.iter_mut() {
            cooldown.tick(time.delta());
            if cooldown.finished() {
                commands.entity(entity).remove::<Cooldown>();
            }
        }
    }
    /// Creates a new cooldown with a given duration in seconds.
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}
