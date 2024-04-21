/// Sparse grid flow for path finding.
use crate::prelude::*;
use bevy::utils::HashMap;

pub mod astar;
pub mod debug;

pub use {astar::AStarRunner, debug::NavigationVisualizerPlugin};

/// Plugin for flow-based navigation.
pub struct NavigationPlugin;
impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NavigationCostEvent>()
            .insert_resource(NavigationGrid2::default())
            .add_systems(FixedUpdate, (NavigationGrid2::resize_on_change,))
            .add_plugins(NavigationVisualizerPlugin);
    }
}

/// Communicates cost updates to the visualizer
#[derive(Event)]
pub struct NavigationCostEvent {
    pub rowcol: RowCol,
    pub cost: f32,
}

/// Sparse storage for flow vectors.
#[derive(Default, DerefMut, Deref, Clone)]
pub struct SparseFlowGrid2(SparseGrid2<Force>);
impl SparseFlowGrid2 {
    /// Compute the weighted force for flow from a single cell.
    pub fn flow_force(&self, position: Vec2, rowcol: RowCol) -> Force {
        if let Some(&force) = self.get(rowcol) {
            // Weight each neighboring force by width - distance.
            let weight = {
                let cell_center = self.to_world_position(rowcol);
                (self.spec.width * self.spec.width - cell_center.distance_squared(position)).max(0.)
            };
            return force * weight;
        }
        Force::ZERO
    }

    /// Adds the amount of force needed to bring the velocity to the underlying flow value.
    pub fn flow_force5(&self, position: Vec2) -> Force {
        let mut total_force = Force::ZERO;

        let rowcol = self.to_rowcol(position);

        total_force += self.flow_force(position, rowcol);
        if self.is_boundary(rowcol) {
            return Force::ZERO;
        }
        // Add forces from neighboring cells.
        for (neighbor_rowcol, _) in self.neighbors8(rowcol) {
            if self.is_boundary(neighbor_rowcol) {
                continue;
            }
            total_force += self.flow_force(position, neighbor_rowcol);
        }
        // What do we need to add to velocity to get total_acccel?
        // v + x = ta
        // x = ta - v
        Force(total_force.normalize_or_zero())
    }
}

pub struct NavigationGrid2Entry {
    pub grid: SparseFlowGrid2,
    pub a_star_runner: AStarRunner,
}
impl NavigationGrid2Entry {
    /// Add a waypoint given rowcols.
    pub fn compute_flow(
        &mut self,
        destination: RowCol,
        sources: &[RowCol],
        obstacles: &Grid2<Obstacle>,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        // TODO: consider if we should also add neighboring cells for each source.
        // let mut expanded_sources: Vec<RowCol> = Vec::with_capacity(sources.len());
        // for &rowcol in &sources {
        //     for neighbor_rowcol in self.grid.get_in_radius_discrete(rowcol, 2) {
        //         sources.push(neighbor_rowcol);
        //     }
        // }

        let costs = self
            .a_star_runner
            .a_star(sources, destination, &self.grid, obstacles);

        // Compute flow direction.
        for (&rowcol, &cost) in &costs {
            let mut min_neighbor_rowcol = rowcol;
            let mut min_neighbor_cost = cost;
            for (neighbor_rowcol, _) in self.grid.neighbors8(rowcol) {
                // Cornering checks for diagonals.
                if neighbor_rowcol.0 != rowcol.0 && neighbor_rowcol.1 != rowcol.1 {
                    if obstacles[(neighbor_rowcol.0, rowcol.1)] != Obstacle::Empty {
                        // info!("Corner skip! {:?} -> {:?}", rowcol, neighbor_rowcol);
                        continue;
                    }
                    if obstacles[(rowcol.0, neighbor_rowcol.1)] != Obstacle::Empty {
                        // info!("Corner skip! {:?} -> {:?}", rowcol, neighbor_rowcol);
                        continue;
                    }
                }
                if let Some(&neighbor_cost) = costs.get(&neighbor_rowcol) {
                    if neighbor_cost < min_neighbor_cost {
                        min_neighbor_rowcol = neighbor_rowcol;
                        min_neighbor_cost = neighbor_cost;
                    }
                }
            }
            self.grid
                .cells
                .insert(rowcol, Force(rowcol.signed_delta8(min_neighbor_rowcol)));
            event_writer.send(NavigationCostEvent { rowcol, cost });
        }
    }
}

/// Mapping from goal RowCol to a sparse flow grid with forces towards that RowCol.
#[derive(Default, Resource, DerefMut, Deref)]
pub struct NavigationGrid2(HashMap<RowCol, NavigationGrid2Entry>);

/// Stores a flow grid per targeted entity.
impl NavigationGrid2 {
    // Resize all grids when spec is updated.
    pub fn resize_on_change(spec: Res<GridSpec>, mut grid: ResMut<Self>) {
        if spec.is_changed() {
            for (
                _entity,
                NavigationGrid2Entry {
                    grid,
                    a_star_runner: _,
                },
            ) in grid.iter_mut()
            {
                grid.resize_with(spec.clone());
            }
        }
    }

    /// Compute navigation for from all sources to the destination.
    pub fn compute_flow(
        &mut self,
        destination: RowCol,
        sources: &[RowCol],
        obstacles: &Grid2<Obstacle>,
        spec: &GridSpec,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        if let Some(nav) = self.get_mut(&destination) {
            let sources: Vec<RowCol> = sources
                .iter()
                .copied()
                .filter(|source| nav.grid.get(*source).is_none())
                .collect();
            if !sources.is_empty() {
                nav.compute_flow(destination, &sources, obstacles, event_writer)
            }
        } else {
            self.insert(
                destination,
                NavigationGrid2Entry {
                    a_star_runner: AStarRunner::new(destination),
                    grid: SparseFlowGrid2(SparseGrid2 {
                        spec: spec.clone(),
                        ..default()
                    }),
                },
            );
        }
    }
}
