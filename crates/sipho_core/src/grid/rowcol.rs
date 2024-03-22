use crate::prelude::*;

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
