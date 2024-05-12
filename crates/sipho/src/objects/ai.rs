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
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Component, Debug, Reflect)]
pub struct EnemyAI {}

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
        mut query: Query<(&mut ZooidHead, Entity, &Team, &mut Consumer), With<EnemyAI>>,
        mut objective_query: Query<&mut Objectives>,
        attached_to: Query<&AttachedTo>,
        positions: Query<&Position>,
        mut commands: ObjectCommands,
        mut elastic_events: EventWriter<SpawnElasticEvent>,
    ) {
        for (mut head, entity, team, mut consumer) in query.iter_mut() {
            let leaves = get_all_leaves(entity, &attached_to);
            let useful_objective = leaves
                .iter()
                .map(|x| objective_query.get(*x).unwrap())
                .find(|x| !matches!(x.last(), Objective::Idle));
            // Force all of the arms to have useful objects if it can.
            if let Some(useful_objective) = useful_objective {
                let useful_objective = useful_objective.clone();
                for leaf in leaves {
                    let mut objective = objective_query.get_mut(leaf).unwrap();
                    if objective.last() == &Objective::Idle {
                        objective.push(useful_objective.last().clone())
                    }
                }
            }
            let entity = head.get_next_limb(entity, &attached_to);

            let position = positions.get(entity).unwrap();
            if consumer.consumed > 0 {
                consumer.consumed -= 1;
                let direction = Vec2::Y;
                let spawn_velocity: Vec2 = direction;
                head.make_linked(
                    &Velocity(spawn_velocity),
                    &mut elastic_events,
                    position,
                    team,
                    Object::Worker,
                    &mut commands,
                    entity,
                );
            }
        }
    }
}
