use std::ops::RangeInclusive;

use crate::prelude::*;
use bevy::{prelude::*, render::render_resource::ShaderType};

/// Represents (row, col) coordinates in the grid.
pub type RowCol = (u16, u16);

/// Extension trait to allow computing distances between RowCols.
pub trait RowColDistance {
    fn distance8(self, other: Self) -> f32;
    fn signed_delta8(self, other: Self) -> Vec2;
    fn above(self) -> Self;
    fn above_right(self) -> Self;
    fn right(self) -> Self;
    fn below_right(self) -> Self;
    fn below(self) -> Self;
    fn below_left(self) -> Self;
    fn left(self) -> Self;
    fn above_left(self) -> Self;
}
impl RowColDistance for RowCol {
    /// Distance on a grid with 8-connectivity in cell space.
    fn distance8(self, rowcol2: Self) -> f32 {
        let (row1, col1) = self;
        let (row2, col2) = rowcol2;

        let dx = col2.abs_diff(col1);
        let dy = row2.abs_diff(row1);
        let diagonals = dx.min(dy);
        let straights = dx.max(dy) - diagonals;
        2f32.sqrt() * diagonals as f32 + straights as f32
    }

    fn above(self) -> Self {
        (self.0 + 1, self.1)
    }
    fn above_right(self) -> Self {
        (self.0 + 1, self.1 + 1)
    }
    fn right(self) -> Self {
        (self.0, self.1 + 1)
    }
    fn below_right(self) -> Self {
        (self.0 - 1, self.1 + 1)
    }
    fn below(self) -> Self {
        (self.0 - 1, self.1)
    }
    fn below_left(self) -> Self {
        (self.0 - 1, self.1 - 1)
    }
    fn left(self) -> Self {
        (self.0, self.1 - 1)
    }
    fn above_left(self) -> Self {
        (self.0 + 1, self.1 - 1)
    }
    /// Signed delta between to rowcol as a float in cell space.
    fn signed_delta8(self, rowcol2: Self) -> Vec2 {
        let (row1, col1) = self;
        let (row2, col2) = rowcol2;
        Vec2 {
            x: (col2 as i16 - col1 as i16) as f32,
            y: (row2 as i16 - row1 as i16) as f32,
        }
    }
}

/// Shader supported grid size.
#[derive(Default, ShaderType, Clone)]
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
            rows: 10,
            cols: 10,
            width: 10.0,
            visualize: true,
            visualize_navigation: false,
        }
    }
}
impl GridSpec {
    pub fn discretize(&self, value: f32) -> u16 {
        (value / self.width) as u16
    }
    // Covert row, col to a single index.
    pub fn flat_index(&self, rowcol: RowCol) -> usize {
        let (row, col) = rowcol;
        row as usize * self.cols as usize + col as usize
    }

    /// Returns (row, col) from a position in world space.
    pub fn to_rowcol(&self, mut position: Vec2) -> RowCol {
        position += self.offset();
        (self.discretize(position.y), self.discretize(position.x))
    }

    /// Returns the world position of the cell coordinate.
    pub fn to_world_position(&self, rowcol: RowCol) -> Vec2 {
        let (row, col) = rowcol;
        Vec2 {
            x: (col as f32 + 0.5) * self.width,
            y: (row as f32 + 0.5) * self.width,
        } - self.offset()
    }

    /// Convert local position [-0.5, 0.5] to world coordinates.
    pub fn local_to_world_position(&self, position: Vec2) -> Vec2 {
        position * 2. * self.offset()
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
        if row == 0 || row == self.rows - 1 {
            return true;
        }
        if col == 0 || col == self.cols - 1 {
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

        let (min_row, min_col) = self.to_rowcol(aabb.min);
        let (max_row, max_col) = self.to_rowcol(aabb.max);
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                results.push((row, col))
            }
        }
        results
    }

    /// Get in radius.
    pub fn get_in_radius(&self, position: Vec2, radius: f32) -> Vec<RowCol> {
        self.get_in_radius_discrete(self.to_rowcol(position), self.discretize(radius) + 1)
    }

    /// Get in radius, with discrete cell position inputs.
    pub fn get_in_radius_discrete(&self, rowcol: RowCol, radius: u16) -> Vec<RowCol> {
        let (row, col) = rowcol;

        let mut results = Vec::default();
        for other_row in self.cell_range(row, radius) {
            for other_col in self.cell_range(col, radius) {
                let other_rowcol = (other_row, other_col);
                if !Self::in_radius(rowcol, other_rowcol, radius) {
                    continue;
                }
                results.push(other_rowcol)
            }
        }
        results
    }

    /// Returns true if a cell is within the given radius to another cell.
    pub fn in_radius(rowcol: RowCol, other_rowcol: RowCol, radius: u16) -> bool {
        let (row, col) = rowcol;
        let (other_row, other_col) = other_rowcol;
        let row_dist = other_row as i16 - row as i16;
        let col_dist = other_col as i16 - col as i16;
        row_dist * row_dist + col_dist * col_dist < radius as i16 * radius as i16
    }

    /// Returns a range starting at `center - radius` ending at `center + radius`.
    fn cell_range(&self, center: u16, radius: u16) -> RangeInclusive<u16> {
        let (min, max) = (
            (center as i16 - radius as i16).max(0) as u16,
            (center + radius).min(self.rows),
        );
        min..=max
    }
}
