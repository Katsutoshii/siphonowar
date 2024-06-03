use std::collections::VecDeque;

use crate::prelude::*;
use bevy::utils::{Entry, HashMap, HashSet};

use super::elastic::SpawnElasticEvent;
use super::zooid_worker::ZooidWorker;
use super::Object;
use super::{ObjectCommands, ObjectSpec, Team};
pub struct ZooidHeadPlugin;
impl Plugin for ZooidHeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                (
                    ZooidHead::spawn,
                    ZooidHead::update,
                    ZooidHead::spawn_linked_zooids,
                )
                    .chain()
                    .in_set(FixedUpdateStage::Spawn),
                // NearestZooidHead::update.in_set(FixedUpdateStage::PostSpawn),
            )
                .in_set(GameStateSet::Running),
        )
        .add_systems(OnExit(GameState::Loading), ZooidHead::spawn_initial);
    }
}

enum SpawnedType {
    Worker,
    Shocker,
}

/// State for a head.
#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct ZooidHead {
    pub spawn_index: usize,
}
impl ZooidHead {
    // Increase head size based on consumer.
    pub fn update(
        mut query: Query<(&mut Transform, &Consumer), With<ZooidHead>>,
        configs: Res<ObjectConfigs>,
    ) {
        let config = configs.get(&Object::Head).unwrap();
        for (mut transform, consumer) in query.iter_mut() {
            let count = 1. + consumer.consumed as f32 / 20.;

            transform.scale = Vec3::splat(config.radius * 1.5 * count / (count + 1.))
        }
    }

    pub fn spawn_initial(mut commands: ObjectCommands, config: Res<TeamConfig>) {
        commands.spawn(ObjectSpec {
            object: Object::Head,
            position: Vec2::ZERO,
            selected: true,
            team: config.player_team,
            ..default()
        });
        for _ in 0..20 {
            commands.spawn(ObjectSpec {
                object: Object::Food,
                position: Vec2::ZERO,
                ..default()
            });
        }
    }

    pub fn spawn(
        mut commands: ObjectCommands,
        config: Res<TeamConfig>,
        obj_config: Res<ObjectConfigs>,
        mut control_events: EventReader<ControlEvent>,
        query: Query<(&ZooidWorker, Entity), With<Selected>>,
    ) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::Head) {
                commands.spawn(ObjectSpec {
                    object: Object::Head,
                    position: control_event.position,
                    team: config.player_team,
                    ..default()
                });
                for _ in 0..20 {
                    commands.spawn(ObjectSpec {
                        object: Object::Food,
                        position: control_event.position,
                        ..default()
                    });
                }
            }
            if control_event.is_pressed(ControlAction::Fuse) {
                info!("Fusing!");
                let mut killable = vec![];
                for (_, entity) in query.iter() {
                    killable.push(entity);

                    if killable.len() >= obj_config.get(&Object::Head).unwrap().spawn_cost as usize
                    {
                        commands.spawn(ObjectSpec {
                            object: Object::Head,
                            position: control_event.position,
                            team: config.player_team,
                            ..default()
                        });
                        for entity in killable.into_iter() {
                            commands.deferred_despawn(entity);
                        }
                        break;
                    }
                }
            }
        }
    }

    // Runs BFS to find the last entity of the shortest limb.
    // Index goes from [0, len]. When 0, we spawn off of the head.
    pub fn get_next_limb(
        &mut self,
        entity: Entity,
        attached_to: &Query<&AttachedTo>,
    ) -> (Entity, usize) {
        let max_arms = 10;
        let head_attached_to = attached_to.get(entity).unwrap();
        if head_attached_to.len() < max_arms {
            return (entity, 0);
        }
        // BFS to find the nearest leaf.
        let mut visited = HashSet::<Entity>::new();
        let mut queue = VecDeque::<(Entity, usize)>::new();
        queue.push_front((entity, 0));
        while let Some((entity, depth)) = queue.pop_front() {
            if !visited.insert(entity) {
                continue;
            }
            if let Ok(attached_to) = attached_to.get(entity) {
                let next: VecDeque<Entity> = attached_to
                    .iter()
                    .filter(|&entity| !visited.contains(entity))
                    .copied()
                    .collect();
                if next.is_empty() {
                    return (entity, depth);
                } else {
                    queue.extend(next.into_iter().map(|x| (x, depth + 1)));
                }
            }
        }
        (entity, 0)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn make_linked(
        &mut self,
        velocity: &Velocity,
        elastic_events: &mut EventWriter<SpawnElasticEvent>,
        audio: &mut EventWriter<AudioEvent>,
        position: &Position,
        team: &Team,
        object: Object,
        commands: &mut ObjectCommands,
        entity: Entity,
    ) {
        if let Some(entity_commands) = commands.spawn(ObjectSpec {
            position: position.0 + velocity.0,
            velocity: Some(*velocity),
            team: *team,
            object,
            // objectives: Objectives::new(Objective::FollowEntity(head_id)),
            ..default()
        }) {
            audio.send(AudioEvent {
                sample: AudioSample::RandomBubble,
                position: Some(position.0),
            });
            elastic_events.send(SpawnElasticEvent {
                elastic: Elastic((entity, entity_commands.id())),
                team: *team,
            });
        }
    }

    pub fn get_shortest_leg_length() {}

    /// System to spawn zooids on Z key.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_linked_zooids(
        mut query: Query<(&mut Self, Entity, &Velocity, &Team), With<Selected>>,
        attachments: Query<&AttachedTo>,
        positions: Query<&Position>,
        mut commands: ObjectCommands,
        configs: Res<ObjectConfigs>,
        mut control_events: EventReader<ControlEvent>,
        mut elastic_events: EventWriter<SpawnElasticEvent>,
        mut audio: EventWriter<AudioEvent>,
    ) {
        let config = configs.get(&Object::Worker).unwrap();
        for control_event in control_events.read() {
            let spawn_type = if control_event.is_pressed(ControlAction::Grow) {
                SpawnedType::Worker
            } else if control_event.is_pressed(ControlAction::SpawnShocker) {
                SpawnedType::Shocker
            } else {
                continue;
            };
            for (mut head, head_id, velocity, team) in query.iter_mut() {
                // Find the shortest leg to spawn an entity onto.
                // Spawn first entity.
                let (entity, _) = head.get_next_limb(head_id, &attachments);

                let position = positions.get(entity).unwrap();
                let food_needed = match spawn_type {
                    SpawnedType::Shocker => 3,
                    SpawnedType::Worker => 1,
                };
                if commands.try_consume(head_id, food_needed).is_ok() {
                    let direction = if let Some(normalized) = velocity.try_normalize() {
                        normalized
                    } else {
                        Vec2::Y
                    };
                    let spawn_velocity: Vec2 = direction * config.spawn_velocity;
                    Self::make_linked(
                        &mut head,
                        &Velocity(spawn_velocity),
                        &mut elastic_events,
                        &mut audio,
                        position,
                        team,
                        match spawn_type {
                            SpawnedType::Shocker => Object::Shocker,
                            SpawnedType::Worker => Object::Worker,
                        },
                        &mut commands,
                        entity,
                    );
                }
            }
            break;
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct NearestZooidHead {
    pub entity: Option<Entity>,
}
impl NearestZooidHead {
    /// Each worker tracks its nearest head.
    pub fn update(
        mut query: Query<(&mut Self, &Team, &Position), Without<ZooidHead>>,
        heads: Query<(Entity, &Team, &Position), With<ZooidHead>>,
    ) {
        let mut team_heads: HashMap<Team, HashMap<Entity, Vec2>> = HashMap::default();
        for (entity, team, position) in &heads {
            let entry = match team_heads.entry(*team) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(HashMap::default()),
            };
            entry.insert(entity, position.0);
        }
        for (mut nearest_head, team, position) in &mut query {
            if let Some(heads) = team_heads.get(team) {
                if let Some(entity) = nearest_head.entity {
                    if !heads.contains_key(&entity) {
                        nearest_head.entity = None;
                    }
                } else {
                    let (entity, _) = heads
                        .iter()
                        .max_by(|(_, p1), (_, p2)| {
                            let d1 = position.distance_squared(**p1);
                            let d2 = position.distance_squared(**p2);
                            d1.partial_cmp(&d2).unwrap()
                        })
                        .unwrap();
                    nearest_head.entity = Some(*entity);
                }
            }
        }
    }
}
