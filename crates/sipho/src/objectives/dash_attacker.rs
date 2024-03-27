use std::time::Duration;

use rand::Rng;

use crate::{objects::CollidingNeighbors, prelude::*};

pub struct DashAttackerPlugin;
impl Plugin for DashAttackerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DashAttacker>().add_systems(
            FixedUpdate,
            DashAttacker::update
                .in_set(SystemStage::PostCompute)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashAttackerState {
    Init,
    Attacking,
    Cooldown,
    Stunned,
}

/// Dash attacker does a periodic dash towards the target.
/// When attacking,
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct DashAttacker {
    pub target: Vec2,
    pub timer: Timer,
    pub state: DashAttackerState,
}
impl Default for DashAttacker {
    fn default() -> Self {
        Self {
            target: Vec2::ZERO,
            timer: Timer::new(Self::attack_delay(), TimerMode::Repeating),
            state: DashAttackerState::Init,
        }
    }
}
impl DashAttacker {
    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(800..1000))
        // Duration::from_millis(800)
    }

    /// Gets the attack duration.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(150)
    }

    /// Gets the attack duration.
    pub fn attack_duration() -> Duration {
        Duration::from_millis(150)
    }

    pub fn next_state(&mut self, in_radius: bool) -> DashAttackerState {
        if !in_radius {
            self.timer.set_duration(Self::attack_cooldown());
            return DashAttackerState::Init;
        }
        match self.state {
            DashAttackerState::Init | DashAttackerState::Attacking | DashAttackerState::Stunned => {
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
            &mut DashAttacker,
            &mut Acceleration,
            &GlobalTransform,
            &CollidingNeighbors,
        )>,
        time: Res<Time>,
        configs: Res<ObjectConfigs>,
        mut damage_events: EventWriter<DamageEvent>,
    ) {
        for (entity, object, velocity, mut attacker, mut acceleration, transform, collisions) in
            query.iter_mut()
        {
            let config = configs.get(object).unwrap();

            attacker.timer.tick(time.delta());

            let delta = attacker.target - transform.translation().xy();

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
                    });
                } else {
                    *acceleration +=
                        Acceleration(delta.normalize_or_zero() * config.attack_velocity);
                }
            }
        }
    }
}
