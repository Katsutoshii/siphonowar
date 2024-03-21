use bevy::{prelude::*, utils::HashSet};

use crate::prelude::*;

/// Stores a set of entities in each grid cell.
pub type EntitySet = HashSet<Entity>;

/// Component to track an entity in the grid.
/// Holds its cell position so it can move/remove itself from the grid.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GridEntity {
    pub cell: Option<RowCol>,
}
impl GridEntity {
    pub fn update(
        mut query: Query<(Entity, &mut Self, &Transform)>,
        mut grid: ResMut<Grid2<EntitySet>>,
        mut event_writer: EventWriter<EntityGridEvent>,
    ) {
        for (entity, mut grid_entity, transform) in &mut query {
            if let Some(event) =
                grid.update_entity(entity, grid_entity.cell, transform.translation.xy())
            {
                grid_entity.cell = event.cell;
                event_writer.send(event);
            }
        }
    }
}

/// Communicates updates to the grid to other systems.
#[derive(Event)]
pub struct EntityGridEvent {
    pub entity: Entity,
    pub prev_cell: Option<RowCol>,
    pub prev_cell_empty: bool,
    pub cell: Option<RowCol>,
}
impl Default for EntityGridEvent {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            prev_cell: None,
            prev_cell_empty: false,
            cell: Some((0, 0)),
        }
    }
}

impl Grid2<EntitySet> {
    /// Update an entity's position in the grid.
    pub fn update_entity(
        &mut self,
        entity: Entity,
        cell: Option<RowCol>,
        position: Vec2,
    ) -> Option<EntityGridEvent> {
        let rowcol = self.to_rowcol(position);

        // Remove this entity's old position if it was different.
        let mut prev_cell: Option<RowCol> = None;
        let mut prev_cell_empty: bool = false;
        if let Some(prev_rowcol) = cell {
            // If in same position, do nothing.
            if prev_rowcol == rowcol {
                return None;
            }

            if let Some(entities) = self.get_mut(prev_rowcol) {
                entities.remove(&entity);
                prev_cell = Some(prev_rowcol);
                prev_cell_empty = entities.is_empty();
            }
        }

        if let Some(entities) = self.get_mut(rowcol) {
            entities.insert(entity);
            return Some(EntityGridEvent {
                entity,
                prev_cell,
                prev_cell_empty,
                cell: Some(rowcol),
            });
        }
        None
    }

    pub fn get_entities_in_radius(&self, position: Vec2, radius: f32) -> HashSet<Entity> {
        let mut other_entities: HashSet<Entity> = HashSet::default();
        let positions = self.get_in_radius(position, radius);
        for rowcol in positions {
            if self.in_bounds(rowcol) {
                other_entities.extend(&self[rowcol]);
            }
        }
        other_entities
    }
    /// Remove an entity from the grid entirely.
    pub fn remove(&mut self, entity: Entity, grid_entity: &GridEntity) {
        if let Some(rowcol) = grid_entity.cell {
            if let Some(cell) = self.get_mut(rowcol) {
                cell.remove(&entity);
            } else {
                error!("No cell at {:?}.", rowcol)
            }
        } else {
            error!("No row col for {:?}", entity)
        }
    }

    /// Get all entities in a given bounding box.
    pub fn get_entities_in_aabb(&self, aabb: &Aabb2) -> Vec<Entity> {
        let mut result = HashSet::default();

        for rowcol in self.get_in_aabb(aabb) {
            if let Some(set) = self.get(rowcol) {
                result.extend(set.iter());
            }
        }
        result.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::{entity::EntitySet, Grid2, GridSpec};

    use bevy::prelude::*;

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
