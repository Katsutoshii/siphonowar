use std::collections::VecDeque;
use std::f32::consts::PI;

use bevy::utils::HashSet;
use rand::Rng;

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
    yeet_timer: Timer,
    yeet_dash_timer: Timer,
    rotation: Vec2,
}

impl Default for EnemyAI {
    fn default() -> EnemyAI {
        EnemyAI {
            free_workers: HashSet::new(),
            clear_objectives_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            yeet_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            yeet_dash_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            rotation: Vec2::from_angle(1.0),
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
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut query: Query<(&mut ZooidHead, Entity, &Team, &mut EnemyAI, &Velocity)>,
        mut objective_query: Query<&mut Objectives>,
        attached_to: Query<&AttachedTo>,
        positions: Query<&Position>,
        mut forces: Query<&mut Force>,
        mut commands: ObjectCommands,
        mut elastic_events: EventWriter<SpawnElasticEvent>,
        mut audio: EventWriter<AudioEvent>,
        time: Res<Time>,
    ) {
        for (mut head, head_entity, team, mut ai, velocity) in query.iter_mut() {
            let leaves = get_all_leaves(head_entity, &attached_to);
            ai.clear_objectives_timer.tick(time.delta());
            if ai.yeet_timer.finished() {
                // kick the head
                if !ai.yeet_dash_timer.finished() {
                    ai.yeet_dash_timer.tick(time.delta());
                    let apply_force = Force(ai.rotation.rotate(velocity.0).normalize_or_zero());
                    *forces.get_mut(head_entity).unwrap() += apply_force * 0.5;
                    for &entity in ai.free_workers.iter() {
                        if let Ok(mut force) = forces.get_mut(entity) {
                            *force += apply_force * 0.5;
                        }
                    }
                } else {
                    ai.yeet_dash_timer.reset();
                    ai.yeet_timer.reset();
                    ai.rotation =
                        Vec2::from_angle(rand::thread_rng().gen_range(-PI / 4.0..PI / 4.0));
                }
            } else {
                ai.yeet_timer.tick(time.delta());
            }
            if ai.clear_objectives_timer.finished() {
                for leaf in ai.free_workers.iter().chain(leaves.iter()) {
                    if let Ok(mut objective) = objective_query.get_mut(*leaf) {
                        objective.clear();
                        objective.push(Objective::Idle);
                    }
                }
            }
            ai.free_workers.retain(|x| positions.contains(*x));
            let active_workers = ai
                .free_workers
                .iter()
                .filter(|&x| attached_to.get(*x).unwrap().len() <= 1);
            // Apply free worker useful objectives
            let useful_objective = active_workers
                .clone()
                .chain(leaves.iter())
                .map(|x| objective_query.get(*x).unwrap())
                .find(|x| !matches!(x.last(), Objective::Idle));
            // Force all of the arms to have useful objects if it can.
            if let Some(useful_objective) = useful_objective {
                let useful_objective = useful_objective.clone();
                for leaf in active_workers {
                    let mut objective = objective_query.get_mut(*leaf).unwrap();
                    if objective.last() == &Objective::Idle {
                        objective.pop();
                        objective.push(useful_objective.last().clone())
                    }
                }
            }

            let (entity, arm_length) = head.get_next_limb(head_entity, &attached_to);
            let position = positions.get(entity).unwrap();
            let direction = Vec2::Y;
            let spawn_velocity: Vec2 = direction;
            if commands.try_consume(head_entity, 1).is_ok() {
                if arm_length < 7 {
                    if let Some(new_entity) = head.make_linked(
                        &Velocity(spawn_velocity),
                        &mut elastic_events,
                        &mut audio,
                        position,
                        team,
                        Object::Worker,
                        &mut commands,
                        entity,
                    ) {
                        ai.free_workers.insert(new_entity);
                    }
                    if let Ok(mut objective) = objective_query.get_mut(entity) {
                        objective.clear();
                        objective.push(Objective::Idle);
                    }
                } else {
                    let position = positions.get(head_entity).unwrap();
                    if let Some(new_entity) = commands.spawn(ObjectSpec {
                        position: Position(position.0 + spawn_velocity),
                        velocity: Some(Velocity(spawn_velocity)),
                        team: *team,
                        object: if ai.free_workers.len() % 20 == 0 {
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
