use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use std::ops::{Index, IndexMut};

use super::GridSpec;

/// 2D Grid containing arbitrary data.
#[derive(Clone, Default, Debug, Deref, DerefMut)]
pub struct SparseGrid2<T: Sized + Default + Clone> {
    #[deref]
    pub spec: GridSpec,
    pub cells: HashMap<RowCol, T>,
}
impl<T: Sized + Default + Clone> Index<RowCol> for SparseGrid2<T> {
    type Output = T;
    fn index(&self, i: RowCol) -> &Self::Output {
        &self.cells[&i]
    }
}
impl<T: Sized + Default + Clone> IndexMut<RowCol> for SparseGrid2<T> {
    fn index_mut(&mut self, i: RowCol) -> &mut T {
        self.cells.get_mut(&i).unwrap()
    }
}
impl<T: Sized + Default + Clone + Send + Sync + 'static> SparseGrid2<T> {
    /// Resize the grid to match the given spec.
    pub fn resize_with(&mut self, spec: GridSpec) {
        self.spec = spec;
    }

    pub fn get(&self, rowcol: RowCol) -> Option<&T> {
        self.cells.get(&rowcol)
    }

    pub fn get_mut(&mut self, rowcol: RowCol) -> Option<&mut T> {
        self.cells.get_mut(&rowcol)
    }
}
