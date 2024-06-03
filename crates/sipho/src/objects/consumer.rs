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
    pub indicators: Vec<Entity>,
}
impl Consumer {
    pub fn new(consumed: usize) -> Self {
        Self {
            consumed,
            ..default()
        }
    }

    pub fn spend(&mut self, n: usize, commands: &mut Commands) {
        for _ in 0..n {
            if let Some(id) = self.indicators.pop() {
                commands.entity(id).despawn();
                self.consumed -= 1;
            }
        }
    }

    pub fn update(
        mut query: Query<(
            Entity,
            &mut Consumer,
            &mut Mass,
            &Position,
            &EnemyCollisions,
            &Transform,
        )>,
        mut damage_events: EventWriter<DamageEvent>,
        mut audio: EventWriter<AudioEvent>,
        mut commands: Commands,
        assets: Res<ObjectAssets>,
    ) {
        for (entity, mut consumer, mut mass, position, colliders, transform) in query.iter_mut() {
            for neighbor in colliders.iter() {
                if neighbor.object == Object::Food {
                    consumer.consumed += 1;
                    let consumed = consumer.consumed as f32;
                    let min_radius = 3.0;
                    let max_radius = 30.0;
                    let radius = min_radius.lerp(max_radius, (consumed / 30.).min(1.));
                    let child_position =
                        radius * Vec2::from_angle(consumer.consumed as f32 * 5.25).normalize();
                    let indicator = commands
                        .spawn(PbrBundle {
                            mesh: assets.object_meshes[&Object::Food].clone(),
                            material: assets.food_material.clone(),
                            transform: Transform {
                                translation: child_position.extend(15.0) / transform.scale,
                                scale: transform.scale.recip() * 6.,
                                ..default()
                            },
                            ..default()
                        })
                        .id();
                    consumer.indicators.push(indicator);
                    commands.entity(entity).add_child(indicator);
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
