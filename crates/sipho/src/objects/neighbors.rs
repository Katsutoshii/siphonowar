use bevy::utils::{FloatOrd, HashSet};

use crate::prelude::*;

pub struct NeighborsPlugin;
impl Plugin for NeighborsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (update
                .in_set(SystemStage::FindNeighbors)
                .in_set(GameStateSet::Running),),
        );
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Neighbor {
    pub entity: Entity,
    pub object: Object,
    pub team: Team,
    pub delta: Vec2,
    pub distance_squared: f32,
}

pub const MAX_NEIGHBORS: usize = 16;

/// Max neighbors: 16
#[derive(Component, Deref, DerefMut, Default)]
pub struct EnemyNeighbors(pub ArrayVec<Neighbor, MAX_NEIGHBORS>);

/// Max neighbors: 16
#[derive(Component, Deref, DerefMut, Default)]
pub struct AlliedNeighbors(pub ArrayVec<Neighbor, MAX_NEIGHBORS>);

/// Max neighbors: 16
#[derive(Component, Deref, DerefMut, Default)]
pub struct CollidingNeighbors(pub ArrayVec<Neighbor, MAX_NEIGHBORS>);

#[derive(Bundle, Default)]
pub struct NeighborsBundle {
    allies: AlliedNeighbors,
    enemies: EnemyNeighbors,
    collisions: CollidingNeighbors,
    grid_entity: GridEntity,
}

pub fn get_neighbors(
    entity: Entity,
    position: Vec2,
    other_entities: &HashSet<Entity>,
    others: &Query<(&Object, &Team, &GlobalTransform)>,
    config: &ObjectConfig,
) -> Vec<Neighbor> {
    let mut neighbors: Vec<Neighbor> = Vec::with_capacity(other_entities.len());

    for &other_entity in other_entities.iter() {
        if entity == other_entity {
            continue;
        }
        if let Ok((other_object, other_team, other_transform)) = others.get(other_entity) {
            let other_position = other_transform.translation().xy();
            let delta = other_position - position;
            let distance_squared = delta.length_squared();
            if distance_squared > config.neighbor_radius * config.neighbor_radius {
                continue;
            }
            neighbors.push(Neighbor {
                entity: other_entity,
                object: *other_object,
                team: *other_team,
                delta,
                distance_squared,
            });
        } else {
            warn!("Missing entity! {:?}", other_entity);
        }
    }

    neighbors.sort_by_key(|neighbor| FloatOrd(neighbor.distance_squared));
    neighbors.truncate(MAX_NEIGHBORS);
    neighbors
}

pub fn update(
    mut query: Query<(
        Entity,
        &mut EnemyNeighbors,
        &mut AlliedNeighbors,
        &mut CollidingNeighbors,
        &Object,
        &Team,
        &GlobalTransform,
    )>,
    others: Query<(&Object, &Team, &GlobalTransform)>,
    grid: Res<Grid2<TeamEntitySets>>,
    configs: Res<ObjectConfigs>,
) {
    query.par_iter_mut().for_each(
        |(
            entity,
            mut enemy_neighbors,
            mut allied_neighbors,
            mut colliding_neighbors,
            object,
            team,
            transform,
        )| {
            let config = configs.get(object).unwrap();
            let position = transform.translation().xy();
            let other_entities_prefetch =
                grid.get_entities_in_radius(position, config.neighbor_radius / 2., &Team::ALL);
            let other_entities = if other_entities_prefetch.len() >= MAX_NEIGHBORS {
                other_entities_prefetch
            } else {
                grid.get_entities_in_radius(position, config.neighbor_radius, &Team::ALL)
            };

            enemy_neighbors.clear();
            allied_neighbors.clear();
            colliding_neighbors.clear();

            let all_neighbors = get_neighbors(entity, position, &other_entities, &others, config);

            for neighbor in all_neighbors.into_iter() {
                if *team == neighbor.team {
                    allied_neighbors.push(neighbor);
                } else {
                    enemy_neighbors.push(neighbor);
                    let other_config = configs.get(&neighbor.object).unwrap();
                    if neighbor.distance_squared
                        < config.radius.powi(2) + other_config.radius.powi(2)
                    {
                        colliding_neighbors.push(neighbor);
                    }
                }
            }
        },
    )
}
