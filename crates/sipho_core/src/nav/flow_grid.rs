/// Sparse grid flow for path finding.
use crate::prelude::*;
use bevy::{
    prelude::*,
    utils::{Entry, HashMap},
};

use super::{AStarRunner, SparseGrid2};

/// Plugin for flow-based navigation.
pub struct NavigationGridPlugin;
impl Plugin for NavigationGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NavigationCostEvent>()
            .add_event::<CreateWaypointEvent>()
            .insert_resource(NavigationGrid2::default())
            .add_systems(
                FixedUpdate,
                (
                    NavigationGrid2::resize_on_change,
                    NavigationGrid2::create_waypoints.in_set(SystemStage::PostApply),
                ),
            );
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
pub struct SparseFlowGrid2(SparseGrid2<Acceleration>);
impl SparseFlowGrid2 {
    /// Compute the weighted acceleration for flow from a single cell.
    pub fn flow_acceleration(&self, position: Vec2, rowcol: RowCol) -> Acceleration {
        if let Some(&acceleration) = self.get(rowcol) {
            // Weight each neighboring acceleration by width - distance.
            let weight = {
                let cell_center = self.to_world_position(rowcol);
                (self.spec.width * self.spec.width - cell_center.distance_squared(position)).max(0.)
            };
            return acceleration * weight;
        }
        Acceleration::ZERO
    }

    /// Adds the amount of acceleration needed to bring the velocity to the underlying flow value.
    pub fn flow_acceleration5(&self, position: Vec2) -> Acceleration {
        let mut total_acceleration = Acceleration::ZERO;

        let rowcol = self.to_rowcol(position);

        total_acceleration += self.flow_acceleration(position, rowcol);
        if self.is_boundary(rowcol) {
            return Acceleration::ZERO;
        }
        // Add accelerations from neighboring cells.
        for (neighbor_rowcol, _) in self.neighbors8(rowcol) {
            if self.is_boundary(neighbor_rowcol) {
                continue;
            }
            total_acceleration += self.flow_acceleration(position, neighbor_rowcol);
        }
        // What do we need to add to velocity to get total_acccel?
        // v + x = ta
        // x = ta - v
        Acceleration(total_acceleration.normalize_or_zero())
    }
}

pub struct NavigationGrid2Entry {
    pub grid: SparseFlowGrid2,
    pub a_star_runner: AStarRunner,
}
impl NavigationGrid2Entry {
    /// Add a waypoint given rowcols.
    pub fn add_waypoint_rowcols(
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
            self.grid.cells.insert(
                rowcol,
                Acceleration(rowcol.signed_delta8(min_neighbor_rowcol)),
            );
            event_writer.send(NavigationCostEvent { rowcol, cost });
        }
    }

    /// Add a waypoint.
    /// Create flows from all points to the waypoint.
    pub fn add_waypoint(
        &mut self,
        event: &CreateWaypointEvent,
        obstacles: &Grid2<Obstacle>,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        let mut sources: Vec<RowCol> = Vec::with_capacity(event.sources.len());
        for &source in &event.sources {
            let rowcol = self.grid.spec.to_rowcol(source);
            for neighbor_rowcol in self.grid.get_in_radius_discrete(rowcol, 2) {
                sources.push(neighbor_rowcol);
            }
        }

        let destination = self.grid.to_rowcol(event.destination);
        self.add_waypoint_rowcols(destination, &sources, obstacles, event_writer);
    }
}

/// Mapping from goal RowCol to a sparse flow grid with accelerations towards that RowCol.
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

    pub fn navigate_to_destination(
        &mut self,
        destination: RowCol,
        sources: &[RowCol],
        obstacles: &Grid2<Obstacle>,
        spec: &GridSpec,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        if let Some(nav) = self.get_mut(&destination) {
            for &source in sources {
                if nav.grid.get(source).is_none() {
                    nav.add_waypoint_rowcols(destination, &[source], obstacles, event_writer)
                }
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

    pub fn create_waypoint(
        &mut self,
        event: &CreateWaypointEvent,
        spec: &GridSpec,
        obstacles: &Grid2<Obstacle>,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        let destination = spec.to_rowcol(event.destination);
        let nav = match self.entry(destination) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(NavigationGrid2Entry {
                a_star_runner: AStarRunner::new(destination),
                grid: SparseFlowGrid2(SparseGrid2 {
                    spec: spec.clone(),
                    ..default()
                }),
            }),
        };
        nav.add_waypoint(event, obstacles, event_writer);
    }

    /// Consumes CreateWaypointEvent events and populates the navigation grid.
    pub fn create_waypoints(
        mut nav_grid: ResMut<Self>,
        mut event_reader: EventReader<CreateWaypointEvent>,
        mut event_writer: EventWriter<NavigationCostEvent>,
        spec: Res<GridSpec>,
        obstacles: Res<Grid2<Obstacle>>,
    ) {
        for event in event_reader.read() {
            nav_grid.create_waypoint(event, &spec, &obstacles, &mut event_writer);
        }
    }
}

/// Event to request waypoint creation.
#[derive(Event, Clone, Debug)]
pub struct CreateWaypointEvent {
    pub destination: Vec2,
    pub sources: Vec<Vec2>,
}
