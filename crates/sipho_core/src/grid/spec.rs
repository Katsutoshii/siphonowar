use std::ops::RangeInclusive;

use crate::prelude::*;
use bevy::{prelude::*, render::render_resource::ShaderType};

/// Shader supported grid size.
#[derive(Default, ShaderType, Debug, Clone)]
#[repr(C)]
pub struct GridSize {
    pub width: f32,
    pub rows: u32,
    pub cols: u32,
}

/// Specification describing how large the grid is.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource)]
pub struct GridSpec {
    pub rows: u16,
    pub cols: u16,
    pub width: f32,
    pub visualize: bool,
    pub visualize_navigation: bool,
}
impl Default for GridSpec {
    fn default() -> Self {
        Self {
            rows: 128,
            cols: 128,
            width: 64.0,
            visualize: true,
            visualize_navigation: false,
        }
    }
}
impl GridSpec {
    pub fn discretize(&self, value: f32) -> Option<u16> {
        if value < 0.0 {
            return None;
        }
        Some((value / self.width) as u16)
    }
    // Covert row, col to a single index.
    pub fn flat_index(&self, rowcol: RowCol) -> usize {
        let (row, col) = rowcol;
        row as usize * self.cols as usize + col as usize
    }

    /// Returns (row, col) from a position in world space.
    pub fn to_rowcol(&self, mut position: Vec2) -> Option<RowCol> {
        position += self.offset();
        let res = (self.discretize(position.y)?, self.discretize(position.x)?);
        if self.in_bounds(res) {
            return Some(res);
        }
        None
    }

    /// Returns the world position of the cell coordinate.
    pub fn to_world_position(&self, rowcol: RowCol) -> Vec2 {
        let (row, col) = rowcol;
        Vec2 {
            x: (col as f32 + 0.5) * self.width,
            y: (row as f32 + 0.5) * self.width,
        } - self.offset()
    }

    /// Convert local position [0, 1] to world coordinates.
    pub fn uv_to_world_position(&self, position: Vec2) -> Vec2 {
        let position = Vec2::new(position.x, 1. - position.y);
        position * self.scale() - self.offset()
    }

    /// Compute the offset vector for this grid spec.
    pub fn offset(&self) -> Vec2 {
        Vec2 {
            x: self.width * self.cols as f32 / 2.,
            y: self.width * self.rows as f32 / 2.,
        }
    }

    /// Compute the (min, max) position for the grid.
    pub fn world2d_bounds(&self) -> Aabb2 {
        Aabb2 {
            min: -self.offset(),
            max: self.offset(),
        }
    }

    /// Compute the (min, max) position for the grid.
    pub fn world2d_bounds_eps(&self) -> Aabb2 {
        Aabb2 {
            min: -self.offset() + self.width,
            max: self.offset() - self.width,
        }
    }

    pub fn scale(&self) -> Vec2 {
        Vec2 {
            x: self.width * self.cols as f32,
            y: self.width * self.rows as f32,
        }
    }

    /// Returns true iff the rowcol is on the boundary of the grid.
    pub fn is_boundary(&self, rowcol: RowCol) -> bool {
        let (row, col) = rowcol;
        let boundary_size = 5;
        if row < boundary_size || row > self.rows - boundary_size {
            return true;
        }
        if col < boundary_size || col > self.cols - boundary_size {
            return true;
        }
        false
    }

    /// Returns true if the rowcol is in bounds.
    pub fn in_bounds(&self, rowcol: RowCol) -> bool {
        let (row, col) = rowcol;
        row < self.rows && col < self.cols
    }

    /// Returns the 8 neighboring cells to the given cell rowcol.
    /// Diagonals have distance sqrt(2).
    pub fn neighbors8(&self, rowcol: RowCol) -> [(RowCol, f32); 8] {
        let (row, col) = rowcol;
        [
            ((row + 1, col - 1), 2f32.sqrt()), // Up left
            ((row + 1, col), 1.),              // Up
            ((row + 1, col + 1), 2f32.sqrt()), // Up right
            ((row, col + 1), 1.),              // Right
            ((row - 1, col + 1), 2f32.sqrt()), // Down right
            ((row - 1, col), 1.),              // Down
            ((row - 1, col - 1), 2f32.sqrt()), // Down left
            ((row, col - 1), 1.),              // Left
        ]
    }

    /// Returns the 4 neighboring cells to the given cell rowcol.
    pub fn neighbors4(&self, rowcol: RowCol) -> [RowCol; 4] {
        let (row, col) = rowcol;
        [
            (row + 1, col), // Up
            (row, col + 1), // Right
            (row - 1, col), // Down
            (row, col - 1), // Left
        ]
    }

    /// Get all cells in a given bounding box.
    pub fn get_in_aabb(&self, aabb: &Aabb2) -> Vec<RowCol> {
        let mut results = Vec::default();

        let min_rowcol = self.to_rowcol(aabb.min);
        let max_rowcol = self.to_rowcol(aabb.max);
        if let (Some((min_row, min_col)), Some((max_row, max_col))) = (min_rowcol, max_rowcol) {
            for row in min_row..=max_row {
                for col in min_col..=max_col {
                    if self.in_bounds((row, col)) {
                        results.push((row, col));
                    }
                }
            }
        }
        results
    }

    /// Get in radius.
    pub fn get_in_radius(&self, position: Vec2, radius: f32) -> Vec<RowCol> {
        if let Some(rowcol) = self.to_rowcol(position) {
            return self.get_in_radius_discrete(rowcol, self.discretize(radius).unwrap() + 1);
        }
        vec![]
    }

    /// Get in radius, with discrete cell position inputs.
    pub fn get_in_radius_discrete(&self, rowcol: RowCol, radius: u16) -> Vec<RowCol> {
        let (row, col) = rowcol;
        if !Self::in_bounds(self, rowcol) {
            return vec![];
        }
        let mut results = Vec::default();
        for other_row in self.cell_range(row, radius) {
            for other_col in self.cell_range(col, radius) {
                let other_rowcol = (other_row, other_col);
                if Self::in_radius(rowcol, other_rowcol, radius)
                    && Self::in_bounds(self, other_rowcol)
                {
                    results.push(other_rowcol)
                }
            }
        }
        results
    }

    /// Returns true if a cell is within the given radius to another cell.
    pub fn in_radius(rowcol: RowCol, other_rowcol: RowCol, radius: u16) -> bool {
        let (row, col) = rowcol;
        let (other_row, other_col) = other_rowcol;
        let row_dist = row.abs_diff(other_row);
        let col_dist = col.abs_diff(other_col);
        row_dist * row_dist + col_dist * col_dist < radius * radius
    }

    /// Returns a range starting at `center - radius` ending at `center + radius`.
    fn cell_range(&self, center: u16, radius: u16) -> RangeInclusive<u16> {
        let (min, max) = (
            (if radius < center { center - radius } else { 0 }),
            (center + radius).min(self.rows),
        );
        min..=max
    }
}
