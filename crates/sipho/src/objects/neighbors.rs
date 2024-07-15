use crate::prelude::*;
use bevy::math::FloatOrd;
use bevy::utils::HashSet;
use enum_iterator::all;
use smallvec::SmallVec;

pub struct NeighborsPlugin;
impl Plugin for NeighborsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AlliedNeighbors>()
            .register_type::<EnemyNeighbors>()
            .register_type::<AlliedCollisions>()
            .register_type::<EnemyCollisions>()
            .add_systems(
                FixedUpdate,
                (update
                    .in_set(FixedUpdateStage::FindNeighbors)
                    .in_set(GameStateSet::Running),),
            );
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Reflect)]
pub struct Neighbor {
    pub entity: Entity,
    pub object: Object,
    pub team: Team,
    pub delta: Vec2,
    pub distance_squared: f32,
}

pub const MAX_NEIGHBORS: usize = 16;
pub const MAX_COLLISIONS: usize = 4;

#[derive(Component, Deref, DerefMut, Default, Reflect)]
#[reflect(Component)]
pub struct EnemyNeighbors(pub SmallVec<[Neighbor; MAX_NEIGHBORS]>);

#[derive(Component, Deref, DerefMut, Default, Reflect)]
#[reflect(Component)]
pub struct AlliedNeighbors(pub SmallVec<[Neighbor; MAX_NEIGHBORS]>);

#[derive(Component, Deref, DerefMut, Default, Reflect)]
#[reflect(Component)]
pub struct AlliedCollisions(pub SmallVec<[Neighbor; MAX_COLLISIONS]>);

#[derive(Component, Deref, DerefMut, Default, Reflect)]
#[reflect(Component)]
pub struct EnemyCollisions(pub SmallVec<[Neighbor; MAX_COLLISIONS]>);

#[derive(Bundle, Default)]
pub struct NeighborsBundle {
    pub allies: AlliedNeighbors,
    pub enemies: EnemyNeighbors,
    pub ally_collisions: AlliedCollisions,
    pub enemy_collisions: EnemyCollisions,
    pub grid_entity: GridEntity,
}

pub fn get_neighbors(
    entity: Entity,
    position: Vec2,
    other_entities: &HashSet<Entity>,
    others: &Query<(&Object, &Team, &Position)>,
    config: &ObjectConfig,
) -> Vec<Neighbor> {
    let mut neighbors: Vec<Neighbor> = Vec::with_capacity(other_entities.len());

    for &other_entity in other_entities.iter() {
        if entity == other_entity {
            continue;
        }
        if let Ok((other_object, other_team, other_position)) = others.get(other_entity) {
            let delta = other_position.0 - position;
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
        &mut EnemyCollisions,
        &Object,
        &Team,
        &Position,
    )>,
    others: Query<(&Object, &Team, &Position)>,
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
            position,
        )| {
            enemy_neighbors.clear();
            allied_neighbors.clear();
            colliding_neighbors.clear();

            let config = configs.get(object).unwrap();
            let ally_entities = grid.get_n_entities_in_radius(
                position.0,
                config.neighbor_radius,
                &[*team],
                MAX_NEIGHBORS,
            );
            for neighbor in
                get_neighbors(entity, position.0, &ally_entities, &others, config).into_iter()
            {
                allied_neighbors.push(neighbor);
                // let other_config = configs.get(&neighbor.object).unwrap();
                // if config.is_colliding(other_config, neighbor.distance_squared) {
                //     colliding_neighbors.push(neighbor);
                // }
            }

            let enemy_teams: Vec<Team> = all::<Team>()
                .filter(|other_team| team != other_team)
                .collect();
            let enemy_entities = grid.get_n_entities_in_radius(
                position.0,
                config.neighbor_radius,
                &enemy_teams,
                MAX_NEIGHBORS,
            );
            for neighbor in
                get_neighbors(entity, position.0, &enemy_entities, &others, config).into_iter()
            {
                enemy_neighbors.push(neighbor);
                let other_config = configs.get(&neighbor.object).unwrap();
                if config.is_colliding(other_config, neighbor.distance_squared)
                    && colliding_neighbors.len() < MAX_COLLISIONS
                {
                    colliding_neighbors.push(neighbor);
                }
            }
        },
    )
}
