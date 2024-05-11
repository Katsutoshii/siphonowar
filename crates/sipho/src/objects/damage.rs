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
            (DamageEvent::update, Health::death)
                .chain()
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running),
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

    /// System for objects dying.
    pub fn death(
        mut objects: Query<(Entity, &Object, &Health, &Position, &Team)>,
        mut object_commands: ObjectCommands,
        mut firework_events: EventWriter<FireworkSpec>,
    ) {
        for (entity, object, health, position, team) in &mut objects {
            if health.health <= 0 {
                object_commands.deferred_despawn(entity);
                firework_events.send(FireworkSpec {
                    size: VfxSize::Medium,
                    position: position.extend(zindex::ZOOIDS_MAX),
                    color: (*team).into(),
                });
                if object == &Object::Plankton {
                    object_commands.spawn(ObjectSpec {
                        object: Object::Food,
                        position: position.0,
                        ..default()
                    });
                }
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct DamageEvent {
    pub damager: Entity,
    pub damaged: Entity,
    pub amount: i32,
    pub velocity: Velocity,
    pub stun: bool,
}
impl DamageEvent {
    pub fn update(
        mut query: Query<(
            Option<&mut DashAttacker>,
            &mut Force,
            &mut Health,
            &Team,
            &Position,
            &mut Objectives,
        )>,
        mut events: EventReader<DamageEvent>,
        mut firework_events: EventWriter<FireworkSpec>,
    ) {
        for event in events.read() {
            let knockback_amount = 3.;
            // Knock back the damager
            if let Ok((_, mut force, _health, _team, _transform, _)) = query.get_mut(event.damager)
            {
                *force += Force(*event.velocity * -1. * knockback_amount);
            }
            // Knock forward the damaged
            if let Ok((_, mut force, _health, _team, _transform, _)) = query.get_mut(event.damaged)
            {
                *force += Force(*event.velocity * 0.5 * knockback_amount);
            }
            // Reduce health and set off firework for the damaged.
            if let Ok((mut attacker, mut _force, mut health, &team, &position, mut objectives)) =
                query.get_mut(event.damaged)
            {
                if health.damageable {
                    if let Some(attacker) = attacker.as_deref_mut() {
                        attacker.state = DashAttackerState::Stunned;
                        attacker.timer.set_duration(Duration::from_secs(0));
                    }
                    health.damage(event.amount);
                };
                let size = VfxSize::Small;

                firework_events.send(FireworkSpec {
                    size,
                    color: team.into(),
                    position: position.extend(zindex::ZOOIDS_MAX),
                });
                if event.stun {
                    if let Objective::Stunned(_) = objectives.last() {
                    } else {
                        objectives.push(Objective::Stunned(Timer::from_seconds(
                            0.5,
                            TimerMode::Once,
                        )));
                    }
                }
            }
        }
    }
}
