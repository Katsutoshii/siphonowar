use bevy::prelude::*;

use crate::prelude::*;

use self::effects::{EffectSize, FireworkSpec};

pub struct DamagePlugin;
impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>().add_systems(
            FixedUpdate,
            (
                Health::update
                    .in_set(SystemStage::Compute)
                    .after(Object::update_objective),
                DamageEvent::update
                    .in_set(SystemStage::Compute)
                    .after(Health::update),
            ),
        );
    }
}

#[derive(Component)]
pub struct Health {
    pub health: i32,
    pub hit_timer: Timer,
}
impl Default for Health {
    fn default() -> Self {
        Self {
            health: 3,
            hit_timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}
impl Health {
    pub fn new(amount: i32) -> Self {
        Self {
            health: amount,
            ..default()
        }
    }
    pub fn damageable(&self) -> bool {
        self.hit_timer.finished()
    }
    pub fn damage(&mut self, amount: i32) {
        self.health -= amount;
        self.hit_timer = Timer::from_seconds(0.5, TimerMode::Once);
    }
    pub fn update(mut query: Query<&mut Health>, time: Res<Time>) {
        for mut health in query.iter_mut() {
            health.hit_timer.tick(time.delta());
        }
    }
}

#[derive(Event)]
pub struct DamageEvent {
    pub damager: Entity,
    pub damaged: Entity,
    pub amount: i32,
    pub velocity: Velocity,
}
impl DamageEvent {
    pub fn update(
        mut query: Query<(&mut Acceleration, &mut Health, &Team, &Transform)>,
        mut events: EventReader<DamageEvent>,
        mut effects: EffectCommands,
    ) {
        for event in events.read() {
            // Knock back the damager
            if let Ok((mut acceleration, _health, _team, _transform)) = query.get_mut(event.damager)
            {
                *acceleration -= Acceleration(event.velocity.0 * 5.);
            }
            // Reduce health and set off firework for the damaged.
            if let Ok((mut acceleration, mut health, &team, &transform)) =
                query.get_mut(event.damaged)
            {
                health.damage(event.amount);
                effects.make_fireworks(FireworkSpec {
                    size: EffectSize::Small,
                    team,
                    transform,
                });
                *acceleration += Acceleration(event.velocity.0 * 2.);
            }
        }
    }
}
