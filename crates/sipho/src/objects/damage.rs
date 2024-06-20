use std::time::Duration;

use crate::{objectives::Stunned, prelude::*};

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
        mut audio: EventWriter<AudioEvent>,
    ) {
        for (entity, object, health, position, team) in &mut objects {
            if health.health <= 0 {
                object_commands.deferred_despawn(entity);
                if object == &Object::Plankton {
                    object_commands.spawn(ObjectSpec {
                        object: Object::Food,
                        position: position.0,
                        ..default()
                    });
                }
                if object != &Object::Food {
                    firework_events.send(FireworkSpec {
                        size: VfxSize::Medium,
                        position: position.extend(zindex::ZOOIDS_MAX),
                        color: (*team).into(),
                    });
                    audio.send(AudioEvent {
                        sample: AudioSample::Punch,
                        position: Some(position.0),
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
        mut query: Query<(Entity, &mut Health, &Team, &Object, &Position)>,
        mut forces: Query<&mut Force>,
        mut events: EventReader<DamageEvent>,
        mut firework_events: EventWriter<FireworkSpec>,
        mut audio_events: EventWriter<AudioEvent>,
        mut commands: Commands,
    ) {
        for event in events.read() {
            let knockback_amount = 3.;
            // Knock back the damager
            if let Ok(mut force) = forces.get_mut(event.damager) {
                *force += Force(*event.velocity * -1. * knockback_amount);
            }
            // Knock forward the damaged
            if let Ok(mut force) = forces.get_mut(event.damaged) {
                *force += Force(*event.velocity * 0.5 * knockback_amount);
            }
            // Reduce health and set off firework for the damaged.
            if let Ok((entity, mut health, &team, object, &position)) = query.get_mut(event.damaged)
            {
                if health.damageable {
                    health.damage(event.amount);
                };

                if object != &Object::Food {
                    let size = VfxSize::Small;
                    firework_events.send(FireworkSpec {
                        size,
                        color: team.into(),
                        position: position.extend(zindex::ZOOIDS_MAX),
                    });
                    audio_events.send(AudioEvent {
                        sample: AudioSample::RandomPop,
                        position: Some(*position),
                        ..default()
                    });
                }
                if event.stun {
                    commands
                        .entity(entity)
                        .insert(Stunned::new(Duration::from_millis(500)));
                }
            }
        }
    }
}
