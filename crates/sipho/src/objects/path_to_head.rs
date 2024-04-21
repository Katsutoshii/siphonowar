use std::collections::VecDeque;

use bevy::utils::HashSet;

use crate::prelude::*;

use super::zooid_head::ZooidHead;

pub struct PathToHeadPlugin;
impl Plugin for PathToHeadPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PathToHead>()
            .register_type::<PathToHeadFollower>()
            .add_systems(
                FixedUpdate,
                (PathToHead::update, PathToHeadFollower::update)
                    .in_set(GameStateSet::Running)
                    .in_set(FixedUpdateStage::AccumulateForces),
            );
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct PathToHead {
    pub head: Option<Entity>,
    pub next: Option<Entity>,
}
impl PathToHead {
    pub fn clear(&mut self) {
        self.head = None;
        self.next = None;
    }

    // Point everything to its head.
    pub fn update(
        mut paths: Query<&mut PathToHead>,
        attachments: Query<&AttachedTo>,
        heads: Query<(Entity, &AttachedTo), With<ZooidHead>>,
    ) {
        for mut path in paths.iter_mut() {
            path.clear();
        }
        for (head_entity, head_attached) in heads.iter() {
            // Run BFS to all attachements.
            let mut queue: VecDeque<Entity> = VecDeque::new();
            let mut visited: HashSet<Entity> = HashSet::new();
            for &entity in head_attached.iter() {
                let mut path = paths.get_mut(entity).unwrap();
                path.head = Some(head_entity);
                path.next = Some(head_entity);
                queue.push_front(entity);
            }
            while let Some(entity) = queue.pop_back() {
                if !visited.insert(entity) {
                    continue;
                }
                let attached = attachments.get(entity).unwrap();

                for &next in attached.iter().filter(|&entity| !visited.contains(entity)) {
                    let mut path = paths.get_mut(next).unwrap();
                    path.head = Some(head_entity);
                    path.next = Some(entity);
                    queue.push_front(next);
                }
            }
        }
    }
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
pub struct PathToHeadFollower {
    pub target: Option<Entity>,
}
impl PathToHeadFollower {
    pub fn update(
        mut query: Query<(
            &Position,
            &Velocity,
            &mut Acceleration,
            &mut PathToHeadFollower,
        )>,
        others: Query<(&Position, &Velocity)>,
    ) {
        // Get the path using follower.
        // Get the target transform using path.
        for (position, velocity, mut acceleration, mut follower) in query.iter_mut() {
            if let Some(target) = follower.target {
                if let Ok((target_position, target_velocity)) = others.get(target) {
                    let delta = target_position.0 - position.0;
                    let velocity_delta = target_velocity.0 - velocity.0;
                    let magnitude = 0.4;
                    *acceleration += Acceleration(delta * magnitude + velocity_delta);
                } else {
                    follower.target = None;
                }
            }
        }
    }
}
