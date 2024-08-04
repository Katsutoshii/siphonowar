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
                (
                    PathToHead::init_heads,
                    PathToHead::update,
                    PathToHeadFollower::update,
                )
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

    pub fn init_heads(
        mut heads: Query<(Entity, &mut PathToHead), (Added<PathToHead>, With<ZooidHead>)>,
    ) {
        for (entity, mut path) in heads.iter_mut() {
            path.head = Some(entity);
        }
    }

    // Point everything to its head.
    pub fn update(
        mut paths: Query<&mut PathToHead, Without<ZooidHead>>,
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
                if let Ok(mut path) = paths.get_mut(entity) {
                    path.head = Some(head_entity);
                    path.next = Some(head_entity);
                    queue.push_front(entity);
                }
            }
            while let Some(entity) = queue.pop_back() {
                if !visited.insert(entity) {
                    continue;
                }
                let attached = attachments.get(entity).unwrap();

                for &next in attached.iter().filter(|&entity| !visited.contains(entity)) {
                    if let Ok(mut path) = paths.get_mut(next) {
                        path.head = Some(head_entity);
                        path.next = Some(entity);
                        queue.push_front(next);
                    }
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
        mut query: Query<(&Position, &Velocity, &mut Force, &mut PathToHeadFollower)>,
        others: Query<(&Position, &Velocity)>,
    ) {
        // Get the path using follower.
        // Get the target transform using path.
        for (position, velocity, mut force, mut follower) in query.iter_mut() {
            if let Some(target) = follower.target {
                if let Ok((target_position, target_velocity)) = others.get(target) {
                    let delta = target_position.0 - position.0;
                    let velocity_delta = target_velocity.0 - velocity.0;
                    let magnitude = 0.3;
                    *force += Force(delta * magnitude + velocity_delta);
                } else {
                    follower.target = None;
                }
            }
        }
    }
}
