use std::collections::VecDeque;

use bevy::utils::HashSet;

use crate::prelude::*;

use crate::objects::elastic::SpawnElasticEvent;
use crate::objects::zooid_head::ZooidHead;
pub struct EnemyAIPlugin;
impl Plugin for EnemyAIPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnemyAI>().add_systems(
            FixedUpdate,
            EnemyAI::update
                .in_set(FixedUpdateStage::AI)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Component, Debug, Reflect)]
pub struct EnemyAI {
    free_workers: HashSet<Entity>,
    clear_objectives_timer: Timer,
}

impl Default for EnemyAI {
    fn default() -> EnemyAI {
        EnemyAI {
            free_workers: HashSet::new(),
            clear_objectives_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

// Runs BFS to find the last entity of the shortest limb.
// Index goes from [0, len]. When 0, we spawn off of the head.
pub fn get_all_leaves(head: Entity, attached_to: &Query<&AttachedTo>) -> Vec<Entity> {
    let head_attached_to = attached_to.get(head).unwrap();
    // BFS to find the first leaf on this limb.
    let mut leaf_entities: Vec<Entity> = vec![];
    let mut visited = HashSet::<Entity>::new();
    let mut queue = VecDeque::<Entity>::new();
    queue.extend(head_attached_to.iter());
    visited.insert(head);

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
                leaf_entities.push(entity);
            } else {
                queue.extend(next.into_iter());
            }
        }
    }
    leaf_entities
}

impl EnemyAI {
    pub fn update(
        mut query: Query<(&mut ZooidHead, Entity, &Team, &mut EnemyAI)>,
        mut objective_query: Query<&mut Objectives>,
        attached_to: Query<&AttachedTo>,
        positions: Query<&Position>,
        mut commands: ObjectCommands,
        mut elastic_events: EventWriter<SpawnElasticEvent>,
        mut audio: EventWriter<AudioEvent>,
        time: Res<Time>,
    ) {
        for (mut head, head_entity, team, mut ai) in query.iter_mut() {
            let leaves = get_all_leaves(head_entity, &attached_to);
            ai.clear_objectives_timer.tick(time.delta());
            if ai.clear_objectives_timer.finished() {
                for leaf in ai.free_workers.iter().chain(leaves.iter()) {
                    let mut objective = objective_query.get_mut(*leaf).unwrap();
                    objective.clear();
                    objective.push(Objective::Idle);
                }
                ai.clear_objectives_timer.reset();
            }
            ai.free_workers.retain(|x| positions.contains(*x));
            // Apply free worker useful objectives
            let useful_objective = ai
                .free_workers
                .iter()
                .chain(leaves.iter())
                .map(|x| objective_query.get(*x).unwrap())
                .find(|x| !matches!(x.last(), Objective::Idle));
            // Force all of the arms to have useful objects if it can.
            if let Some(useful_objective) = useful_objective {
                let useful_objective = useful_objective.clone();
                for leaf in ai.free_workers.iter().chain(leaves.iter()) {
                    let mut objective = objective_query.get_mut(*leaf).unwrap();
                    if objective.last() == &Objective::Idle {
                        objective.pop();
                        objective.push(useful_objective.last().clone())
                    }
                }
            }

            let (entity, arm_length) = head.get_next_limb(head_entity, &attached_to);
            let position = positions.get(entity).unwrap();
            if commands.try_consume(head_entity, 1).is_ok() {
                let direction = Vec2::Y;
                let spawn_velocity: Vec2 = direction;
                if arm_length < 7 {
                    head.make_linked(
                        &Velocity(spawn_velocity),
                        &mut elastic_events,
                        &mut audio,
                        position,
                        team,
                        Object::Worker,
                        &mut commands,
                        entity,
                    );
                    if let Ok(mut objective) = objective_query.get_mut(entity) {
                        objective.clear();
                        objective.push(Objective::Idle);
                    }
                } else if commands.try_consume(head_entity, 3).is_ok() {
                    let position = positions.get(head_entity).unwrap();
                    if let Some(new_entity) = commands.spawn(ObjectSpec {
                        position: position.0 + spawn_velocity,
                        velocity: Some(Velocity(spawn_velocity)),
                        team: *team,
                        object: if ai.free_workers.len() % 6 == 0 {
                            Object::Shocker
                        } else {
                            Object::Worker
                        },
                        // objectives: Objectives::new(Objective::FollowEntity(head_id)),
                        ..default()
                    }) {
                        ai.free_workers.insert(new_entity.id());
                    }
                }
            }
        }
    }
}
