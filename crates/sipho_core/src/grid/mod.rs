use bevy::transform::TransformSystem;

use crate::prelude::*;

pub mod entity;
pub mod fog;
pub mod grid2;
pub mod obstacles;
pub mod rowcol;
pub mod sparse_grid2;
pub mod spec;
pub mod visualizer;

pub use {
    entity::{EntityGridEvent, EntitySet, GridEntity},
    fog::{FogPlugin, VisibilityUpdate, VisibilityUpdateEvent},
    grid2::{Grid2, Grid2Plugin},
    // minimap::{MinimapPlugin, MinimapShaderMaterial},
    obstacles::{Obstacle, ObstaclesPlugin},
    rowcol::{RowCol, RowColDistance},
    sparse_grid2::SparseGrid2,
    spec::{GridSize, GridSpec},
    visualizer::GridVisualizerPlugin,
};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridSpec>()
            .init_resource::<GridSpec>()
            .add_event::<EntityGridEvent>()
            .add_plugins((
                GridVisualizerPlugin,
                // MinimapPlugin,
                ObstaclesPlugin,
                FogPlugin,
                Grid2Plugin::<EntitySet>::default(),
            ))
            .add_systems(
                PostUpdate,
                GridEntity::update
                    .after(TransformSystem::TransformPropagate)
                    .in_set(GameStateSet::Running),
            );
    }
}
