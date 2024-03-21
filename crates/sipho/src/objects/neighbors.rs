use crate::prelude::*;
use bevy::prelude::*;

/// Plugin for running zooids simulation.
pub struct NeighborsPlugin;
impl Plugin for NeighborsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (update.in_set(SystemStage::FindNeighbors),));
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Neighbor {
    pub entity: Entity,
    pub object: Object,
    pub delta: Vec2,
    pub distance_squared: f32,
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct EnemyNeighbors(pub Vec<Neighbor>);
#[derive(Component, Deref, DerefMut, Default)]
pub struct AlliedNeighbors(pub Vec<Neighbor>);

#[derive(Bundle, Default)]
pub struct NeighborsBundle {
    allies: AlliedNeighbors,
    enemies: EnemyNeighbors,
    grid_entity: GridEntity,
}

pub fn update(
    mut query: Query<(
        Entity,
        &mut EnemyNeighbors,
        &mut AlliedNeighbors,
        &Object,
        &Team,
        &GlobalTransform,
    )>,
    others: Query<(&Object, &Team, &GlobalTransform)>,
    grid: Res<Grid2<EntitySet>>,
    configs: Res<Configs>,
) {
    query.par_iter_mut().for_each(
        |(entity, mut enemy_neighbors, mut allied_neighbors, object, team, transform)| {
            let config = &configs.objects[object];
            let position = transform.translation().xy();
            let other_entities = grid.get_entities_in_radius(position, config.neighbor_radius);

            enemy_neighbors.clear();
            enemy_neighbors.reserve_exact(other_entities.len());
            allied_neighbors.clear();
            allied_neighbors.reserve_exact(other_entities.len());

            for other_entity in other_entities {
                if entity == other_entity {
                    continue;
                }
                let (other_object, other_team, other_transform) = others.get(other_entity).unwrap();
                let other_position = other_transform.translation().xy();

                let delta = other_position - position;
                let distance_squared = delta.length_squared();
                if distance_squared > config.neighbor_radius * config.neighbor_radius {
                    continue;
                }

                let neighbor = Neighbor {
                    entity: other_entity,
                    object: *other_object,
                    delta,
                    distance_squared,
                };
                if team == other_team {
                    allied_neighbors.push(neighbor)
                } else {
                    enemy_neighbors.push(neighbor)
                }
            }
        },
    )
}
