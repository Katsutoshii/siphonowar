use crate::prelude::*;
use bevy::prelude::*;

pub mod astar;
pub mod entity;
pub mod fog;
pub mod grid2;
pub mod minimap;
pub mod navigation;
pub mod navigation_visualizer;
pub mod obstacles;
pub mod rowcol;
pub mod shader_plane;
pub mod sparse_grid2;
pub mod spec;
pub mod visualizer;

pub use {
    astar::AStarRunner,
    entity::{EntityGridEvent, EntitySet, GridEntity},
    fog::FogPlugin,
    grid2::{Grid2, Grid2Plugin},
    minimap::{MinimapPlugin, MinimapShaderMaterial},
    navigation::{CreateWaypointEvent, NavigationCostEvent, NavigationGrid2, NavigationPlugin},
    navigation_visualizer::NavigationVisualizerPlugin,
    obstacles::{Obstacle, ObstaclesPlugin},
    rowcol::{RowCol, RowColDistance},
    shader_plane::{ShaderPlaneAssets, ShaderPlaneMaterial},
    sparse_grid2::SparseGrid2,
    spec::{GridSize, GridSpec},
    visualizer::GridVisualizerPlugin,
};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridSpec>()
            .add_event::<EntityGridEvent>()
            .add_plugins((
                GridVisualizerPlugin,
                MinimapPlugin,
                ObstaclesPlugin,
                NavigationPlugin,
                NavigationVisualizerPlugin,
                FogPlugin,
                Grid2Plugin::<EntitySet>::default(),
            ))
            .add_systems(
                FixedUpdate,
                GridEntity::update.in_set(SystemStage::PostApply),
            );
    }
}
