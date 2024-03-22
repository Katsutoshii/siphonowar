use std::time::Duration;

use rand::Rng;

use crate::prelude::*;

pub struct DashAttackerPlugin;
impl Plugin for DashAttackerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DashAttacker>().add_systems(
            FixedUpdate,
            DashAttacker::update.in_set(SystemStage::PostApply),
        );
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashAttackerState {
    Init,
    Attacking,
    Cooldown,
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
    /// Gets a random attack delay.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..100))
    }

    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(500..1000))
    }

    /// Gets the atack duration
    pub fn attack_duration() -> Duration {
        Duration::from_millis(30)
    }

    pub fn next_state(&mut self) {
        self.state = match self.state {
            DashAttackerState::Attacking => {
                self.timer.set_duration(Self::attack_cooldown());
                DashAttackerState::Attacking
            }
            DashAttackerState::Init | DashAttackerState::Cooldown => {
                self.timer.set_duration(Self::attack_duration());
                DashAttackerState::Attacking
            }
        }
    }

    pub fn update(
        mut query: Query<(
            &Object,
            &mut DashAttacker,
            &mut Acceleration,
            &GlobalTransform,
        )>,
        time: Res<Time>,
        configs: Res<ObjectConfigs>,
    ) {
        for (object, mut attacker, mut acceleration, transform) in query.iter_mut() {
            let config = configs.get(object).unwrap();

            attacker.timer.tick(time.delta());

            if attacker.timer.finished() {
                attacker.timer.reset();
                attacker.next_state();
            }

            if attacker.state == DashAttackerState::Attacking {
                let delta = attacker.target - transform.translation().xy();
                *acceleration += Acceleration(delta.normalize() * config.attack_velocity)
            }
        }
    }
}
