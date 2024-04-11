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
    entity::{EntityGridEvent, EntitySet, GridEntity, TeamEntitySets},
    fog::{VisibilityUpdate, VisibilityUpdateEvent},
    grid2::{Grid2, Grid2Plugin},
    obstacles::Obstacle,
    rowcol::{RowCol, RowColDistance},
    sparse_grid2::SparseGrid2,
    spec::{GridSize, GridSpec},
};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridSpec>()
            .init_resource::<GridSpec>()
            .add_event::<EntityGridEvent>()
            .add_plugins((
                visualizer::GridVisualizerPlugin,
                entity::EntityGridPlugin,
                obstacles::ObstaclesPlugin,
                fog::FogPlugin,
            ));
    }
}
