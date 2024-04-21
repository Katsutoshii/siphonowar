use bevy::utils::{Entry, HashMap};

use crate::prelude::*;

pub struct NavigatorPlugin;
impl Plugin for NavigatorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Navigator>().add_systems(
            FixedUpdate,
            (Navigator::update, Navigator::update_force)
                .chain()
                .in_set(FixedUpdateStage::PostPhysics)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct Navigator {
    pub target: Vec2,
    pub slow_factor: f32,
    pub target_radius: f32,
}
impl Navigator {
    pub fn update(
        query: Query<(&Navigator, &Position)>,
        mut grid: ResMut<NavigationGrid2>,
        spec: Res<GridSpec>,
        obstacles: Res<Grid2<Obstacle>>,
        mut event_writer: EventWriter<NavigationCostEvent>,
    ) {
        let mut destinations: HashMap<RowCol, Vec<RowCol>> = HashMap::new();
        for (navigator, position) in query.iter() {
            let source = spec.to_rowcol(position.0);
            let destination = spec.to_rowcol(navigator.target);
            match destinations.entry(destination) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(Vec::with_capacity(1)),
            }
            .push(source);
        }

        // Populate the grid.
        for (&destination, sources) in destinations.iter() {
            grid.compute_flow(destination, sources, &obstacles, &spec, &mut event_writer)
        }

        // Remove old cells where there is no objective leading to that destination.
        let rowcols_to_remove: Vec<RowCol> = grid
            .keys()
            .filter(|&destination| !destinations.contains_key(destination))
            .copied()
            .collect();
        for rowcol in rowcols_to_remove {
            grid.remove(&rowcol);
        }
    }

    pub fn update_force(
        mut query: Query<(
            &Object,
            &Navigator,
            &mut Transform,
            &Position,
            &Velocity,
            &mut Force,
        )>,
        grid: ResMut<NavigationGrid2>,
        configs: Res<ObjectConfigs>,
        spec: Res<GridSpec>,
    ) {
        for (object, navigator, mut transform, position, velocity, mut force) in query.iter_mut() {
            let config = configs.get(object).unwrap();
            let target_rowcol = spec.to_rowcol(navigator.target);

            if let Some(flow_grid) = grid.get(&target_rowcol) {
                let target_cell_center = flow_grid.grid.to_world_position(target_rowcol);
                let flow_force = flow_grid.grid.flow_force5(position.0) * config.nav_flow_factor;
                let slow_force =
                    navigator.slow_force(*velocity, position.0, target_cell_center, flow_force);

                *force += flow_force + slow_force;

                if velocity.length_squared() > 2. {
                    let angle = transform.rotation.z;
                    transform.rotate_z(0.01 * (velocity.to_angle() - angle));
                }
            }
        }
    }

    /// Apply a slowing force against current velocity when near the goal.
    /// Also, undo some of the force force when near the goal.
    pub fn slow_force(
        &self,
        velocity: Velocity,
        position: Vec2,
        target_position: Vec2,
        flow_force: Force,
    ) -> Force {
        let position_delta = target_position - position;
        let dist_squared = position_delta.length_squared();
        let radius_squared = self.target_radius * self.target_radius;

        //  When within radius, this is negative
        let radius_diff = (dist_squared - radius_squared) / radius_squared;
        Force(
            self.slow_factor
                * if dist_squared < radius_squared {
                    -1.0 * velocity.0
                } else {
                    Vec2::ZERO
                }
                + flow_force.0 * radius_diff.clamp(-1., 0.),
        )
    }
}
