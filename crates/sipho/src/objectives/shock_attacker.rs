use rand::Rng;
use std::time::Duration;

use crate::prelude::*;

use super::Navigator;

pub struct ShockAttackerPlugin;
impl Plugin for ShockAttackerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShockAttacker>().add_systems(
            FixedUpdate,
            ShockAttacker::update
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockAttackerState {
    Attacking,
    Cooldown,
    Stunned,
}

/// Dash attacker does a periodic dash towards the target.
/// When attacking,
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct ShockAttacker {
    pub timer: Timer,
    pub state: ShockAttackerState,
}
impl Default for ShockAttacker {
    fn default() -> Self {
        Self {
            timer: Timer::new(Self::attack_delay(), TimerMode::Repeating),
            state: ShockAttackerState::Cooldown,
        }
    }
}
impl ShockAttacker {
    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(1500..2000))
        // Duration::from_millis(800)
    }

    /// Gets the attack duration.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(100..200))
    }

    /// Gets the attack duration.
    pub fn attack_duration() -> Duration {
        Duration::from_millis(100)
    }

    pub fn next_state(&mut self, in_radius: bool) -> ShockAttackerState {
        if !in_radius {
            self.timer.set_duration(Self::attack_cooldown());
            return ShockAttackerState::Cooldown;
        }
        match self.state {
            ShockAttackerState::Attacking | ShockAttackerState::Stunned => {
                self.timer.set_duration(Self::attack_cooldown());
                ShockAttackerState::Cooldown
            }
            ShockAttackerState::Cooldown => {
                self.timer.set_duration(Self::attack_duration());
                ShockAttackerState::Attacking
            }
        }
    }

    /// Check cooldown timers and accelerate when in state Attacking.
    /// Stop attacking after the first hit.
    pub fn update(
        mut query: Query<(
            Entity,
            &Object,
            &mut Velocity,
            &Navigator,
            &mut ShockAttacker,
            &Position,
            &EnemyNeighbors,
        )>,
        time: Res<Time>,
        configs: Res<ObjectConfigs>,
        mut damage_events: EventWriter<DamageEvent>,
        mut lightning: LightningCommands,
        mut firework_events: EventWriter<FireworkSpec>,
        mut audio: EventWriter<AudioEvent>,
    ) {
        for (entity, object, mut velocity, navigator, mut attacker, position, enemies) in
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

            if attacker.state == ShockAttackerState::Attacking {
                if let Some(enemy) = enemies.first() {
                    *velocity = Velocity::ZERO;
                    attacker.state = attacker.next_state(true);
                    let interaction = config.interactions.get(&enemy.object).unwrap();
                    damage_events.send(DamageEvent {
                        damager: entity,
                        damaged: enemy.entity,
                        amount: interaction.damage_amount,
                        velocity: *velocity,
                        stun: true,
                    });
                    lightning.make_lightning(position.0, navigator.target, zindex::ZOOIDS_MAX);
                    firework_events.send(FireworkSpec {
                        position: navigator.target.extend(zindex::ZOOIDS_MAX),
                        color: FireworkColor::White,
                        size: VfxSize::Small,
                    });
                    audio.send(AudioEvent {
                        sample: AudioSample::RandomZap,
                        position: Some(position.0),
                    });
                }
            }
        }
    }
}
