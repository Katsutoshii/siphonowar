use std::time::Duration;

use crate::{objects::EnemyCollisions, prelude::*};

use super::Navigator;

pub struct DashAttackerPlugin;
impl Plugin for DashAttackerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DashAttacker>().add_systems(
            FixedUpdate,
            DashAttacker::update
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashAttackerState {
    Searching,
    Attacking,
}

/// Dash attacker does a periodic dash towards the target.
/// When attacking,
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct DashAttacker {
    pub state: DashAttackerState,
    pub timer: Timer,
}
impl Default for DashAttacker {
    fn default() -> Self {
        Self {
            state: DashAttackerState::Searching,
            timer: Timer::new(Self::ATTACK_DURATION, TimerMode::Repeating),
        }
    }
}
impl DashAttacker {
    pub const ATTACK_DURATION: Duration = Duration::from_millis(150);
    pub const ATTACK_COOLDOWN: Duration = Duration::from_millis(800);

    pub fn reset(&mut self, entity: Entity, commands: &mut Commands) {
        self.timer.reset();
        self.state = DashAttackerState::Searching;
        commands
            .entity(entity)
            .insert(Cooldown::new(Self::ATTACK_COOLDOWN));
    }

    /// Check for in radius.
    /// When NOT on cooldown
    pub fn update(
        mut query: Query<
            (
                Entity,
                &Object,
                &Velocity,
                &Navigator,
                &mut DashAttacker,
                &mut Force,
                &Position,
                &EnemyCollisions,
            ),
            (Without<Cooldown>, Without<Stunned>),
        >,
        configs: Res<ObjectConfigs>,
        time: Res<Time>,
        mut commands: Commands,
        mut damage_events: EventWriter<DamageEvent>,
    ) {
        for (entity, object, velocity, navigator, mut attacker, mut force, position, collisions) in
            query.iter_mut()
        {
            let config = configs.get(object).unwrap();

            let delta = navigator.target - position.0;

            match attacker.state {
                DashAttackerState::Searching => {
                    let distance_squared = delta.length_squared();
                    if distance_squared < config.attack_radius * config.attack_radius {
                        attacker.state = DashAttackerState::Attacking;
                    }
                }
                DashAttackerState::Attacking => {
                    attacker.timer.tick(time.delta());
                    if attacker.timer.finished() {
                        attacker.reset(entity, &mut commands);
                    } else if let Some(collision) = collisions.first() {
                        attacker.reset(entity, &mut commands);
                        let interaction = config.interactions.get(&collision.object).unwrap();
                        damage_events.send(DamageEvent {
                            damager: entity,
                            damaged: collision.entity,
                            amount: interaction.damage_amount,
                            velocity: *velocity,
                            stun: false,
                        });
                    } else {
                        *force += Force(delta.normalize_or_zero() * config.attack_velocity);
                    }
                }
            }
        }
    }
}
