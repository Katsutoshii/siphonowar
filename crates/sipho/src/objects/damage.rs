use std::time::Duration;

use crate::{
    objectives::{dash_attacker::DashAttackerState, DashAttacker},
    prelude::*,
};

pub struct DamagePlugin;
impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>().add_systems(
            FixedUpdate,
            (DamageEvent::update)
                .in_set(SystemStage::Compute)
                .in_set(GameStateSet::Running)
                .after(Object::update_objective),
        );
    }
}

#[derive(Component)]
pub struct Health {
    pub health: i32,
    pub damageable: bool,
}
impl Default for Health {
    fn default() -> Self {
        Self {
            health: 1,
            damageable: true,
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
    pub fn damage(&mut self, amount: i32) {
        self.health -= amount;
    }
}

#[derive(Event, Debug)]
pub struct DamageEvent {
    pub damager: Entity,
    pub damaged: Entity,
    pub amount: i32,
    pub velocity: Velocity,
}
impl DamageEvent {
    pub fn update(
        mut query: Query<(
            Option<&mut DashAttacker>,
            &mut Acceleration,
            &mut Health,
            &Team,
            &GlobalTransform,
        )>,
        mut events: EventReader<DamageEvent>,
        mut effects: EffectCommands,
    ) {
        for event in events.read() {
            let knockback_amount = 3.;
            // Knock back the damager
            if let Ok((_, mut acceleration, _health, _team, _transform)) =
                query.get_mut(event.damager)
            {
                *acceleration += Acceleration(*event.velocity * -1. * knockback_amount);
            }
            // Reduce health and set off firework for the damaged.
            if let Ok((mut attacker, mut _acceleration, mut health, &team, &transform)) =
                query.get_mut(event.damaged)
            {
                let size = if health.damageable {
                    if let Some(attacker) = attacker.as_deref_mut() {
                        attacker.state = DashAttackerState::Stunned;
                        attacker.timer.set_duration(Duration::from_secs(0));
                    }
                    health.damage(event.amount);
                    VfxSize::Small
                } else {
                    VfxSize::Tiny
                };
                effects.make_fireworks(FireworkSpec {
                    size,
                    team,
                    transform: transform.into(),
                });
            }
        }
    }
}
