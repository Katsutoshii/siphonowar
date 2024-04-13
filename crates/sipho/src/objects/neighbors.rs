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
            if config.in_radius(distance_squared) {
                neighbors.push(Neighbor {
                    entity: other_entity,
                    object: *other_object,
                    team: *other_team,
                    delta,
                    distance_squared,
                });
            }
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
            enemy_neighbors.clear();
            allied_neighbors.clear();
            colliding_neighbors.clear();

            let config = configs.get(object).unwrap();
            let position = transform.translation().xy();

            let ally_entities = grid.get_n_entities_in_radius(
                position,
                config.neighbor_radius,
                &[*team],
                MAX_NEIGHBORS,
            );
            for neighbor in
                get_neighbors(entity, position, &ally_entities, &others, config).into_iter()
            {
                allied_neighbors.push(neighbor);
                let other_config = configs.get(&neighbor.object).unwrap();
                if config.is_colliding(other_config, neighbor.distance_squared) {
                    colliding_neighbors.push(neighbor);
                }
            }

            let enemy_teams: Vec<Team> = Team::ALL
                .iter()
                .copied()
                .filter(|other_team| team != other_team)
                .collect();
            let enemy_entities = grid.get_n_entities_in_radius(
                position,
                config.neighbor_radius,
                &enemy_teams,
                MAX_NEIGHBORS,
            );
            for neighbor in
                get_neighbors(entity, position, &enemy_entities, &others, config).into_iter()
            {
                enemy_neighbors.push(neighbor);
                let other_config = configs.get(&neighbor.object).unwrap();
                if config.is_colliding(other_config, neighbor.distance_squared) {
                    colliding_neighbors.push(neighbor);
                }
            }
        },
    )
}
