use std::{
    cmp::Ordering,
    collections::{BTreeSet, BinaryHeap},
};

use bevy::utils::HashMap;

use crate::prelude::*;

use super::navigation::SparseFlowGrid2;

/// State for running A* search to fill out flow cost grid.
/// See https://doc.rust-lang.org/std/collections/binary_heap/index.html#examples
#[derive(Copy, Clone, PartialEq)]
pub struct AStarState {
    rowcol: RowCol,
    cost: f32,
    heuristic: f32,
}
impl AStarState {
    // Priority scoring function f.
    fn f(&self) -> f32 {
        self.cost + self.heuristic
    }
}
impl Eq for AStarState {}
impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f()
            .partial_cmp(&self.f())
            .expect("NaN cost found in A*.")
            .then_with(|| self.rowcol.cmp(&other.rowcol))
    }
}
impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct to allow running A* on demand while re-using old results.
pub struct AStarRunner {
    pub destination: RowCol,
    pub costs: HashMap<RowCol, f32>,
    heap: BinaryHeap<AStarState>,
}
impl AStarRunner {
    /// Initialize AStarRunner to a goal.
    pub fn new(destination: RowCol) -> Self {
        let mut heap = BinaryHeap::new();
        heap.push(AStarState {
            cost: 0.,
            rowcol: destination,
            heuristic: 0.,
        });
        Self {
            destination,
            costs: HashMap::new(),
            heap,
        }
    }

    /// Compute heuristic factor on naive 8-distance heuristic.
    /// We want to use the heuristic more at higher distances from the destination.
    pub fn heuristic_factor(&self, source: RowCol) -> f32 {
        let min_grid_dist = 1.;
        let max_grid_dist = 30.;
        let dist = self.destination.distance8(source);
        let max_heuristic = 0.9;
        let final_dist = dist.clamp(min_grid_dist, max_grid_dist);
        max_heuristic * final_dist / max_grid_dist
    }
    /// Runs A star from the given source to the destination.
    pub fn a_star_from_source(
        &mut self,
        source: RowCol,
        grid: &SparseFlowGrid2,
        obstacles: &Grid2<Obstacle>,
    ) {
        // We're at `start`, with a zero cost
        if grid.is_boundary(self.destination) {
            return;
        }

        let heuristic_factor = self.heuristic_factor(source);

        // Examine the frontier with lower cost nodes first (min-heap)
        while let Some(AStarState {
            cost,
            rowcol,
            heuristic: _,
        }) = self.heap.pop()
        {
            // Skip already finalized cells.
            if self.costs.contains_key(&rowcol) {
                continue;
            }

            // Costs inserted here are guaranteed to be the best costs seen so far.
            self.costs.insert(rowcol, cost);

            // If at the goal, we're done!
            if rowcol == source {
                return;
            }

            // For each node we can reach, see if we can find a way with
            // a lower cost going through this node
            for (neighbor_rowcol, neighbor_cost) in grid.neighbors8(rowcol) {
                // Skip out of bounds positions.
                if grid.is_boundary(neighbor_rowcol) {
                    continue;
                }
                if obstacles[neighbor_rowcol] != Obstacle::Empty {
                    continue;
                }

                self.heap.push(AStarState {
                    cost: cost + neighbor_cost,
                    rowcol: neighbor_rowcol,
                    heuristic: heuristic_factor * neighbor_rowcol.distance8(source),
                });
            }
        }
    }

    /// Run A* search from destination to reach all sources.
    pub fn a_star(
        &self,
        sources: &[RowCol],
        destination: RowCol,
        grid: &SparseFlowGrid2,
        obstacles: &Grid2<Obstacle>,
    ) -> HashMap<RowCol, f32> {
        let sources: BTreeSet<RowCol> = sources
            .iter()
            .copied()
            .filter(|&rowcol| grid.in_bounds(rowcol) && obstacles[rowcol] == Obstacle::Empty)
            .collect();
        let mut runner = AStarRunner::new(destination);
        for source in sources {
            if runner.costs.contains_key(&source) {
                continue;
            }
            runner.a_star_from_source(source, grid, obstacles);
        }
        runner.costs
    }
}
