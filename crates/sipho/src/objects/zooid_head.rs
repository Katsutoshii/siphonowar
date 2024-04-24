use std::collections::VecDeque;

use crate::prelude::*;
use bevy::utils::{Entry, HashMap, HashSet};
use sipho_core::grid::fog::FogConfig;

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
                (ZooidHead::spawn, ZooidHead::spawn_zooids).in_set(FixedUpdateStage::Spawn),
                NearestZooidHead::update.in_set(FixedUpdateStage::PostSpawn),
            )
                .in_set(GameStateSet::Running),
        );
    }
}

/// State for a head.
#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct ZooidHead {
    pub spawn_index: usize,
}
impl ZooidHead {
    pub fn spawn(
        mut commands: ObjectCommands,
        config: Res<TeamConfig>,
        obj_config: Res<ObjectConfigs>,
        mut control_events: EventReader<ControlEvent>,
        query: Query<(&ZooidWorker, Entity, &Selected)>,
    ) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::SpawnHead) {
                commands.spawn(ObjectSpec {
                    object: Object::Head,
                    position: control_event.position,
                    team: config.player_team,
                    ..default()
                });
            }
            if control_event.is_pressed(ControlAction::Fuse) {
                info!("Fusing!");
                let mut killable = vec![];
                for (_, entity, selected) in query.iter() {
                    if selected.is_selected() {
                        killable.push(entity);
                    }
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
    pub fn get_next_limb(&mut self, entity: Entity, attached_to: &Query<&AttachedTo>) -> Entity {
        let max_arms = 10;
        let head_attached_to = attached_to.get(entity).unwrap();
        let num_attachments = head_attached_to.len();
        self.spawn_index += 1;
        if num_attachments < max_arms {
            return entity;
        }
        let start = head_attached_to[self.spawn_index % max_arms.min(num_attachments)];
        // BFS to find the first leaf on this limb.
        let mut visited = HashSet::<Entity>::new();
        let mut queue = VecDeque::<Entity>::new();
        queue.push_front(start);
        visited.insert(entity);
        while let Some(entity) = queue.pop_back() {
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
                    return entity;
                } else {
                    queue.extend(next.into_iter());
                }
            }
        }
        entity
    }

    /// System to spawn zooids on Z key.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_zooids(
        mut query: Query<(
            &mut Self,
            Entity,
            &Velocity,
            &Team,
            &Selected,
            &mut Consumer,
        )>,
        attachments: Query<&AttachedTo>,
        positions: Query<&Position>,
        mut commands: ObjectCommands,
        configs: Res<ObjectConfigs>,
        fog_config: Res<FogConfig>,
        mut control_events: EventReader<ControlEvent>,
        mut elastic_events: EventWriter<SpawnElasticEvent>,
    ) {
        let config = configs.get(&Object::Worker).unwrap();
        for control_event in control_events.read() {
            if !control_event.is_pressed(ControlAction::SpawnZooid) {
                continue;
            }
            for (mut head, head_id, velocity, team, selected, mut consumer) in query.iter_mut() {
                if !selected.is_selected() {
                    continue;
                }

                // Find the shortest leg to spawn an entity onto.
                // Spawn first entity.
                let entity = head.get_next_limb(head_id, &attachments);

                let position = positions.get(entity).unwrap();

                if consumer.consumed > 0 {
                    consumer.consumed -= 1;
                    let direction = if let Some(normalized) = velocity.try_normalize() {
                        normalized
                    } else {
                        Vec2::Y
                    };
                    let spawn_velocity: Vec2 = direction * config.spawn_velocity;
                    if let Some(entity_commands) = commands.spawn(ObjectSpec {
                        position: position.0 + spawn_velocity,
                        velocity: Some(Velocity(spawn_velocity)),
                        team: *team,
                        // objectives: Objectives::new(Objective::FollowEntity(head_id)),
                        ..default()
                    }) {
                        elastic_events.send(SpawnElasticEvent {
                            elastic: Elastic((entity, entity_commands.id())),
                            team: fog_config.player_team,
                        });
                    }
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
