use std::time::Duration;

use rand::Rng;

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
    Attacking,
    Cooldown,
    Stunned,
}

/// Dash attacker does a periodic dash towards the target.
/// When attacking,
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct DashAttacker {
    pub timer: Timer,
    pub state: DashAttackerState,
}
impl Default for DashAttacker {
    fn default() -> Self {
        Self {
            timer: Timer::new(Self::attack_delay(), TimerMode::Repeating),
            state: DashAttackerState::Cooldown,
        }
    }
}
impl DashAttacker {
    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(600..700))
    }

    /// Gets the attack duration.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..100))
    }

    /// Gets the attack duration.
    pub fn attack_duration() -> Duration {
        Duration::from_millis(150)
    }

    pub fn next_state(&mut self, in_radius: bool) -> DashAttackerState {
        if !in_radius {
            self.timer.set_duration(Self::attack_cooldown());
            return DashAttackerState::Cooldown;
        }
        match self.state {
            DashAttackerState::Attacking | DashAttackerState::Stunned => {
                self.timer.set_duration(Self::attack_cooldown());
                DashAttackerState::Cooldown
            }
            DashAttackerState::Cooldown => {
                self.timer.set_duration(Self::attack_duration());
                DashAttackerState::Attacking
            }
        }
    }

    /// Check cooldown timers and accelerate when in state Attacking.
    /// Stop attacking after the first hit.
    pub fn update(
        mut query: Query<(
            Entity,
            &Object,
            &Velocity,
            &Navigator,
            &mut DashAttacker,
            &mut Force,
            &Position,
            &EnemyCollisions,
        )>,
        time: Res<Time>,
        configs: Res<ObjectConfigs>,
        mut damage_events: EventWriter<DamageEvent>,
    ) {
        for (entity, object, velocity, navigator, mut attacker, mut force, position, collisions) in
            query.iter_mut()
        {
            let config = configs.get(object).unwrap();

            attacker.timer.tick(time.delta());

            let delta = navigator.target - position.0;

            if attacker.timer.finished() {
                attacker.timer.reset();
                let distance_squared = delta.length_squared();
                let in_radius = distance_squared < config.attack_radius * config.attack_radius;
                attacker.state = attacker.next_state(in_radius);
            }

            if attacker.state == DashAttackerState::Attacking {
                if let Some(collision) = collisions.first() {
                    attacker.state = attacker.next_state(true);
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
