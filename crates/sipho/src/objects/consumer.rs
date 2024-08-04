use std::f32::consts::PI;

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
    pub food_indicators: Vec<Entity>,
    pub gem_indicators: Vec<Entity>,
}
impl Consumer {
    pub fn new() -> Self {
        Self { ..default() }
    }

    pub fn spend_food(&mut self, n: usize, commands: &mut Commands) {
        for _ in 0..n {
            if let Some(id) = self.food_indicators.pop() {
                commands.entity(id).despawn();
            }
        }
    }

    pub fn spend_gems(&mut self, n: usize, commands: &mut Commands) {
        for _ in 0..n {
            if let Some(id) = self.gem_indicators.pop() {
                commands.entity(id).despawn();
            }
        }
    }

    pub fn food_consumed(&self) -> usize {
        self.food_indicators.len()
    }

    pub fn gems_consumed(&self) -> usize {
        self.food_indicators.len()
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
                if matches!(neighbor.object, Object::Food | Object::Gem) {
                    let consumed = consumer.food_consumed() as f32;
                    let min_radius = 3.0;
                    let max_radius = 30.0;
                    let radius = min_radius.lerp(max_radius, (consumed / 30.).min(1.));
                    let child_position = radius
                        * Vec2::from_angle(consumer.food_consumed() as f32 * PI * 0.6).normalize();
                    let indicator = commands
                        .spawn(PbrBundle {
                            mesh: assets.object_meshes[&neighbor.object].clone(),
                            material: if neighbor.object == Object::Food {
                                assets.food_material.clone()
                            } else {
                                assets.crystal_material.clone()
                            },
                            transform: Transform {
                                translation: child_position.extend(15.0 - radius / 4.)
                                    / transform.scale,
                                scale: transform.scale.recip() * 7.,
                                ..default()
                            },
                            ..default()
                        })
                        .id();
                    if neighbor.object == Object::Food {
                        consumer.food_indicators.push(indicator);
                    }
                    if neighbor.object == Object::Gem {
                        consumer.gem_indicators.push(indicator);
                    }
                    commands.entity(entity).add_child(indicator);
                    audio.send(AudioEvent {
                        sample: AudioSample::RandomBubble,
                        position: Some(position.0),
                        ..default()
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
            let count = (consumer.food_consumed() + consumer.gems_consumed()) as f32;
            *mass = Mass(1.0 + (2. * count / (count + 1.0)));
        }
    }
}
