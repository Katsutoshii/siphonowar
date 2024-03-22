use crate::prelude::*;

pub mod astar;
pub mod debug;
pub mod flow_grid;
pub mod navigator;

pub use {
    astar::AStarRunner,
    flow_grid::{CreateWaypointEvent, NavigationCostEvent, NavigationGrid2, SparseFlowGrid2},
    navigator::{NavTarget, Navigator},
};

pub struct NavigationPlugin;
impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            flow_grid::NavigationGridPlugin,
            debug::NavigationDebugPlugin,
        ));
    }
}
