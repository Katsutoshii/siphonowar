use crate::prelude::*;

use super::neighbors::EnemyCollisions;

pub struct ConsumerPlugin;
impl Plugin for ConsumerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Consumer>().add_systems(
            FixedUpdate,
            Consumer::update
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running)
                .before(DamageEvent::update),
        );
    }
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct Consumer {
    pub consumed: usize,
}
impl Consumer {
    pub fn new(consumed: usize) -> Self {
        Self { consumed }
    }
    pub fn update(
        mut query: Query<(
            Entity,
            &mut Consumer,
            &mut Mass,
            &Position,
            &EnemyCollisions,
        )>,
        mut damage_events: EventWriter<DamageEvent>,
        mut audio: EventWriter<AudioEvent>,
    ) {
        for (entity, mut consumer, mut mass, position, colliders) in query.iter_mut() {
            for neighbor in colliders.iter() {
                if neighbor.object == Object::Food {
                    consumer.consumed += 1;
                    audio.send(AudioEvent {
                        sample: AudioSample::RandomBubble,
                        position: Some(position.0),
                    });
                    damage_events.send(DamageEvent {
                        damager: entity,
                        damaged: neighbor.entity,
                        amount: 1,
                        velocity: Velocity::ZERO,
                        stun: false,
                    });
                }
            }
            let count = consumer.consumed as f32;
            *mass = Mass(1.0 + (2. * count / (count + 1.0)));
        }
    }
}
