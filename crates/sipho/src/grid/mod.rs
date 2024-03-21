use crate::prelude::*;
use bevy::prelude::*;

mod spec;
pub use spec::{GridSize, GridSpec, RowCol, RowColDistance};
mod fog;
pub use fog::FogPlugin;
mod entity;
mod visualizer;
pub use entity::{EntityGridEvent, EntitySet, GridEntity};
mod obstacles;
pub use obstacles::{Obstacle, ObstaclesPlugin};
mod grid2;
pub use grid2::Grid2;
mod sparse_grid2;
pub use sparse_grid2::SparseGrid2;
mod shader_plane;
pub use shader_plane::{ShaderPlaneAssets, ShaderPlaneMaterial};
mod astar;
pub use astar::AStarRunner;
mod navigation;
pub use navigation::{CreateWaypointEvent, NavigationCostEvent, NavigationGrid2};
mod minimap;
pub use minimap::{MinimapPlugin, MinimapShaderMaterial};
mod navigation_visualizer;

pub use self::{
    grid2::Grid2Plugin, navigation::NavigationPlugin,
    navigation_visualizer::NavigationVisualizerPlugin, visualizer::GridVisualizerPlugin,
};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridSpec>()
            .add_event::<EntityGridEvent>()
            .add_plugins(GridVisualizerPlugin)
            .add_plugins(MinimapPlugin)
            .add_plugins(ObstaclesPlugin)
            .add_plugins(NavigationPlugin)
            .add_plugins(NavigationVisualizerPlugin)
            .add_plugins(FogPlugin)
            .add_plugins(Grid2Plugin::<EntitySet>::default())
            .add_systems(
                FixedUpdate,
                GridEntity::update.in_set(SystemStage::PostApply),
            );
    }
}
#[cfg(test)]
mod tests {
    use crate::GridSpec;

    #[test]
    fn grid_radius() {
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (2, 2);
            let radius = 2;
            assert!(GridSpec::in_radius(
                (row, col),
                (other_row, other_col),
                radius
            ));
        }
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (4, 4);
            let radius = 2;
            assert!(!GridSpec::in_radius(
                (row, col),
                (other_row, other_col),
                radius
            ));
        }
    }
}
