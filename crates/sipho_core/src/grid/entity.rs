use std::ops::{Index, IndexMut};

use crate::prelude::*;
use bevy::{asset::DependencyLoadState, prelude::*, utils::HashSet};

pub struct EntityGridPlugin;
impl Plugin for EntityGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Grid2Plugin::<TeamEntitySets>::default())
            .add_systems(
                FixedUpdate,
                (
                    GridEntity::update,
                    GridEntity::drop_oobs,
                    GridEntity::cleanup,
                )
                    .chain()
                    .in_set(FixedUpdateStage::PreDespawn)
                    .in_set(GameStateSet::Running),
            );
    }
}
/// Stores a set of entities in each grid cell.
pub type EntitySet = SmallSet<[Entity; 8]>;

#[derive(Default, Clone, Deref, DerefMut, Debug)]
pub struct TeamEntitySets([EntitySet; Team::COUNT]);
impl Index<Team> for TeamEntitySets {
    type Output = EntitySet;
    fn index(&self, i: Team) -> &Self::Output {
        &self.0[i as usize]
    }
}
impl IndexMut<Team> for TeamEntitySets {
    fn index_mut(&mut self, i: Team) -> &mut Self::Output {
        &mut self.0[i as usize]
    }
}

/// Component to track an entity in the grid.
/// Holds its cell position so it can move/remove itself from the grid.
#[derive(Component, Reflect, Default, Copy, Clone)]
#[reflect(Component)]
pub struct GridEntity {
    pub rowcol: Option<RowCol>,
}
impl GridEntity {
    pub fn drop_oobs(
        query: Query<(Entity, &Position)>,
        grid: ResMut<Grid2<TeamEntitySets>>,
        mut despawns_writer: EventWriter<DespawnEvent>,
    ) {
        for (entity, position) in query.iter() {
            if !grid.in_bounds(grid.to_rowcol(position.0)) {
                despawns_writer.send(DespawnEvent(entity));
            }
        }
    }
    pub fn update(
        mut query: Query<(Entity, &mut Self, &Team, &Position)>,
        mut grid: ResMut<Grid2<TeamEntitySets>>,
        mut event_writer: EventWriter<EntityGridEvent>,
    ) {
        for (entity, mut grid_entity, team, transform) in &mut query {
            let rowcol = grid.to_rowcol(transform.0);
            if let Some(event) = grid.update(entity, *team, grid_entity.rowcol, rowcol) {
                grid_entity.rowcol = event.rowcol;
                event_writer.send(event);
            }
        }
    }
    pub fn cleanup(
        query: Query<(Entity, &Self, &Team)>,
        mut grid: ResMut<Grid2<TeamEntitySets>>,
        mut despawns: EventReader<DespawnEvent>,
        mut grid_events: EventWriter<EntityGridEvent>,
    ) {
        for despawn_event in despawns.read() {
            let (entity, grid_entity, team) = query.get(despawn_event.0).unwrap();
            if let Some(rowcol) = grid_entity.rowcol {
                if let Some(grid_event) = grid.remove(entity, *team, rowcol) {
                    grid_events.send(grid_event);
                } else {
                    error!("No rowcol for {:?}", entity)
                }
            }
        }
    }
}
/// Communicates updates to the grid to other systems.
#[derive(Event, Debug)]
pub struct EntityGridEvent {
    pub entity: Entity,
    pub team: Team,
    pub prev_rowcol: Option<RowCol>,
    pub prev_empty: bool,
    pub rowcol: Option<RowCol>,
}
impl Default for EntityGridEvent {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            team: Team::None,
            prev_rowcol: None,
            prev_empty: false,
            rowcol: Some((0, 0)),
        }
    }
}

impl Grid2<TeamEntitySets> {
    /// Update an entity's position in the grid.
    pub fn update(
        &mut self,
        entity: Entity,
        team: Team,
        prev_rowcol: Option<RowCol>,
        rowcol: RowCol,
    ) -> Option<EntityGridEvent> {
        // Remove this entity's old position if it was different.
        let mut prev_empty: bool = false;
        if let Some(prev_rowcol) = prev_rowcol {
            // If in same position, do nothing.
            if prev_rowcol == rowcol {
                return None;
            }

            if let Some(entities) = self.get_mut(prev_rowcol) {
                entities[team].remove(&entity);
                prev_empty = entities[team].is_empty();
            }
        }

        if let Some(entities) = self.get_mut(rowcol) {
            entities[team].insert(entity);
            return Some(EntityGridEvent {
                entity,
                team,
                prev_rowcol,
                prev_empty,
                rowcol: Some(rowcol),
            });
        }
        None
    }

    pub fn get_entities_in_radius(
        &self,
        position: Vec2,
        radius: f32,
        teams: &[Team],
    ) -> HashSet<Entity> {
        let mut other_entities: HashSet<Entity> = HashSet::default();
        let positions = self.get_in_radius(position, radius);
        for rowcol in positions {
            if self.in_bounds(rowcol) {
                for &team in teams {
                    other_entities.extend(self[rowcol][team].iter());
                }
            }
        }
        other_entities
    }

    /// Get entities in radius, first checking half radius and returning early if that gives enough entities.
    pub fn get_n_entities_in_radius(
        &self,
        position: Vec2,
        radius: f32,
        teams: &[Team],
        n: usize,
    ) -> HashSet<Entity> {
        let prefetch = self.get_entities_in_radius(position, radius / 2., teams);
        if prefetch.len() >= n {
            prefetch
        } else {
            self.get_entities_in_radius(position, radius, teams)
        }
    }

    /// Remove an entity from the grid entirely.
    pub fn remove(
        &mut self,
        entity: Entity,
        team: Team,
        rowcol: RowCol,
    ) -> Option<EntityGridEvent> {
        if let Some(entities) = self.get_mut(rowcol) {
            let team_entities = &mut entities[team];
            team_entities.remove(&entity);
            return Some(EntityGridEvent {
                entity,
                team,
                prev_rowcol: Some(rowcol),
                prev_empty: team_entities.is_empty(),
                rowcol: None,
            });
        } else {
            error!("No cell at {:?}.", rowcol)
        }
        None
    }

    /// Get all entities in a given bounding box.
    pub fn get_entities_in_aabb(&self, aabb: &Aabb2) -> Vec<Entity> {
        let mut result = HashSet::default();

        for rowcol in self.get_in_aabb(aabb) {
            if let Some(entities) = self.get(rowcol) {
                for team_entities in entities.iter() {
                    result.extend(team_entities.iter());
                }
            }
        }
        result.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_update() {
        let mut grid = Grid2::<EntitySet> {
            spec: GridSpec {
                rows: 10,
                cols: 10,
                width: 10.0,
                visualize: false,
                visualize_navigation: false,
            },
            ..Default::default()
        };
        grid.resize();
        assert_eq!(grid.spec.offset(), Vec2 { x: 50.0, y: 50.0 });
        let rowcol = grid.spec.to_rowcol(Vec2 { x: 0., y: 0. });
        assert_eq!(rowcol, (5, 5));

        assert!(grid.get_mut((5, 5)).is_some());
        assert!(grid.get((5, 5)).is_some());
    }
}
