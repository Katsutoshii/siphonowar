use crate::prelude::*;
use bevy::utils::{Entry, HashMap};
use rand::Rng;
use std::time::Duration;

pub mod config;
pub mod debug;

pub use {config::ObjectiveConfig, debug::ObjectiveDebugger};

pub struct ObjectivePlugin;
impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ObjectiveConfig>()
            .register_type::<Objectives>()
            .register_type::<Vec<Objective>>()
            .register_type::<Objective>()
            .add_systems(
                FixedUpdate,
                ((
                    Objectives::update_waypoints,
                    Objectives::update,
                    ObjectiveDebugger::update,
                )
                    .chain()
                    .in_set(SystemStage::PostApply),),
            );
    }
}

#[derive(Debug, Clone)]
// Entity will attack nearest enemy in surrounding grid
pub struct AttackEntity {
    pub entity: Entity,
    pub frame: u16,
    pub cooldown: Timer,
}

/// Represents the objective of the owning entity.
#[derive(Component, Default, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub enum Objective {
    /// Entity has no objective.
    #[default]
    None,
    /// Entity wants to follow the transform of another entity.
    FollowEntity(Entity),
    /// Attack Entity
    AttackEntity {
        entity: Entity,
        frame: u16,
        cooldown: Timer,
    },
}
impl Objective {
    /// Given an objective, get the next one (if there should be a next one, else None).
    pub fn try_attacking(&self, entity: Entity) -> Option<Self> {
        match self {
            Self::None | Self::FollowEntity(_) => Some(Self::AttackEntity {
                entity,
                frame: 0,
                cooldown: Timer::from_seconds(
                    Self::attack_delay().as_secs_f32(),
                    TimerMode::Repeating,
                ),
            }),
            Self::AttackEntity { .. } => None,
        }
    }

    /// Gets a random attack delay.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..100))
    }

    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(500..1000))
    }

    /// Resolves an objective.
    pub fn resolve(
        &mut self,
        transform: &GlobalTransform,
        query: &Query<(&GlobalTransform, Option<&Velocity>)>,
        time: &Time,
        config: &ObjectiveConfig,
    ) -> ResolvedObjective {
        match self {
            Self::None => ResolvedObjective::None,
            Self::FollowEntity(entity) => {
                if let Ok((other_transform, _other_velocity)) = query.get(*entity) {
                    ResolvedObjective::FollowEntity {
                        entity: *entity,
                        position: other_transform.translation().xy(),
                    }
                } else {
                    warn!("Invalid entity for follow!");
                    warn!("{:?}", query.get(*entity));
                    ResolvedObjective::None
                }
            }
            Self::AttackEntity {
                entity,
                frame,
                cooldown,
            } => {
                cooldown.tick(time.delta());
                if let Ok((other_transform, other_velocity)) = query.get(*entity) {
                    let position = transform.translation().xy();
                    let other_position = other_transform.translation().xy();
                    let target_position = other_position
                        + if let Some(velocity) = other_velocity {
                            velocity.0
                        } else {
                            Vec2::ZERO
                        };
                    let delta = target_position - position;
                    if delta.length_squared() < config.attack_radius * config.attack_radius
                        && cooldown.finished()
                    {
                        cooldown.set_duration(Self::attack_cooldown());
                        *frame = 3;
                    }
                    if *frame > 0 {
                        *frame -= 1;
                    }
                    ResolvedObjective::AttackEntity {
                        entity: *entity,
                        position,
                        target_position,
                        frame: *frame,
                    }
                } else {
                    ResolvedObjective::None
                }
            }
        }
    }

    /// If this objective is following an entity, return that Entity.
    pub fn get_followed_entity(&self) -> Option<Entity> {
        match self {
            Self::AttackEntity { entity, .. } => Some(*entity),
            Self::FollowEntity(entity) => Some(*entity),
            Self::None => None,
        }
    }
}
/// Represents the objectives of the owning entity.
/// The stack always has Objective::None at the bottom.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Objectives(Vec<Objective>);
impl Default for Objectives {
    fn default() -> Self {
        Self(vec![Objective::None])
    }
}
impl Objectives {
    /// Construct an objective with default to None (idle).
    pub fn new(objective: Objective) -> Self {
        Self(vec![Objective::None, objective])
    }
    /// Get the last objective.
    pub fn last(&self) -> &Objective {
        unsafe { self.0.get_unchecked(self.0.len() - 1) }
    }
    /// Get the last objective.
    pub fn last_mut(&mut self) -> &mut Objective {
        let i = self.0.len() - 1;
        unsafe { self.0.get_unchecked_mut(i) }
    }
    /// Resets the objectives.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Push an objective on the stack.
    pub fn push(&mut self, objective: Objective) {
        self.0.push(objective)
    }
    /// Pop an objective, but only if it's not the bottom None objective.
    pub fn pop(&mut self) -> Option<Objective> {
        if self.0.len() > 1 {
            self.0.pop()
        } else {
            None
        }
    }

    // Start attacking
    pub fn start_attacking(&mut self, entity: Entity) {
        if let Some(objective) = self.last().try_attacking(entity) {
            self.push(objective);
        }
    }

    /// Update acceleration from the current objective.
    pub fn update(
        mut query: Query<(
            &mut Self,
            &Object,
            &GlobalTransform,
            &Velocity,
            &mut Acceleration,
        )>,
        others: Query<(&GlobalTransform, Option<&Velocity>)>,
        configs: Res<ObjectConfigs>,
        grid_spec: Res<GridSpec>,
        navigation_grid: Res<NavigationGrid2>,
        obstacles_grid: Res<Grid2<Obstacle>>,
        time: Res<Time>,
    ) {
        for (mut objectives, object, transform, velocity, mut acceleration) in &mut query {
            if *object == Object::Food {
                continue;
            }
            let config = configs.get(object).unwrap();
            let obstacles_acceleration = obstacles_grid
                .obstacles_acceleration(transform.translation().xy(), *velocity)
                * config.obstacle_acceleration;
            *acceleration += obstacles_acceleration;
            let resolved = objectives.resolve(transform, &others, &time, &config.objective);
            *acceleration +=
                resolved.acceleration(transform, *velocity, config, &grid_spec, &navigation_grid);
        }
    }

    /// Resolve the entity references for the objective and store them in ResolvedObjective.
    /// If there are invalid entity references (deleted entities), remove those objectives.
    pub fn resolve(
        &mut self,
        transform: &GlobalTransform,
        query: &Query<(&GlobalTransform, Option<&Velocity>)>,
        time: &Time,
        config: &ObjectiveConfig,
    ) -> ResolvedObjective {
        while self.last() != &Objective::None {
            let resolved = self.last_mut().resolve(transform, query, time, config);
            if resolved != ResolvedObjective::None {
                return resolved;
            }
            self.0.pop();
        }
        ResolvedObjective::None
    }

    /// Also create new ones for moved waypoints.
    pub fn update_waypoints(
        all_objectives: Query<(Entity, &Objectives), Without<Waypoint>>,
        transforms: Query<&GlobalTransform>,
        mut grid: ResMut<NavigationGrid2>,
        obstacles: Res<Grid2<Obstacle>>,
        spec: Res<GridSpec>,
        mut event_writer: EventWriter<NavigationCostEvent>,
    ) {
        // All active destinations to their current sources.
        let mut destinations: HashMap<RowCol, Vec<RowCol>> = HashMap::new();
        for (entity, objectives) in all_objectives.iter() {
            if let Some(followed_entity) = objectives.last().get_followed_entity() {
                let source_rowcol = if let Ok(source_transform) = transforms.get(entity) {
                    spec.to_rowcol(source_transform.translation().xy())
                } else {
                    continue;
                };
                if let Ok(destination_transform) = transforms.get(followed_entity) {
                    let destination_rowcol =
                        spec.to_rowcol(destination_transform.translation().xy());
                    let value = match destinations.entry(destination_rowcol) {
                        Entry::Occupied(o) => o.into_mut(),
                        Entry::Vacant(v) => v.insert(Vec::with_capacity(1)),
                    };
                    value.push(source_rowcol)
                }
            }
        }

        // Populate any cells that haven't been computed yet.
        for (&destination, sources) in &destinations {
            grid.navigate_to_destination(
                destination,
                sources,
                &obstacles,
                &spec,
                &mut event_writer,
            );
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
}

/// Represents the objective of the owning entity.
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub enum ResolvedObjective {
    /// Entity has no objective.
    #[default]
    None,
    /// Entity wants to follow the transform of another entity.
    FollowEntity { entity: Entity, position: Vec2 },
    /// Attack Entity
    AttackEntity {
        entity: Entity,
        position: Vec2,
        target_position: Vec2,
        frame: u16,
    },
}
impl ResolvedObjective {
    // Returns acceleration for this objective.
    pub fn acceleration(
        &self,
        transform: &GlobalTransform,
        velocity: Velocity,
        config: &ObjectConfig,
        grid_spec: &GridSpec,
        navigation_grid: &NavigationGrid2,
    ) -> Acceleration {
        let position = transform.translation().xy();
        match self {
            Self::FollowEntity {
                entity: _,
                position: target_position,
            } => Self::accelerate_to_position(
                position,
                *target_position,
                config,
                velocity,
                grid_spec,
                navigation_grid,
                /*slow_factor=*/ 1.0,
            ),
            Self::AttackEntity {
                entity: _,
                position,
                target_position,
                frame,
            } => {
                let delta = *target_position - *position;
                if *frame > 0 {
                    Acceleration(delta.normalize() * config.attack_velocity)
                } else {
                    Self::accelerate_to_position(
                        *position,
                        *target_position,
                        config,
                        velocity,
                        grid_spec,
                        navigation_grid,
                        /*slow_factor=*/ 0.5,
                    ) + Acceleration(delta.normalize() * 0.0)
                }
            }
            // If no objective, slow down.
            Self::None => {
                let idle_slow_threshold = config.idle_speed;
                let velocity_squared = velocity.length_squared();
                if velocity_squared == 0. {
                    return Acceleration::ZERO;
                }
                let slow_magnitude =
                    (velocity_squared - idle_slow_threshold).max(0.) / velocity_squared;
                let slow_vector = -velocity.0 * slow_magnitude;
                Acceleration(slow_vector)
            }
        }
    }

    // Returns acceleration for following an entity.
    pub fn accelerate_to_position(
        position: Vec2,
        target_position: Vec2,
        config: &ObjectConfig,
        velocity: Velocity,
        grid_spec: &GridSpec,
        navigation_grid: &NavigationGrid2,
        slow_factor: f32,
    ) -> Acceleration {
        let target_cell = grid_spec.to_rowcol(target_position);
        if let Some(nav) = navigation_grid.get(&target_cell) {
            let target_cell_position = nav.grid.to_world_position(target_cell);
            let flow_acceleration = nav.grid.flow_acceleration5(position) * config.nav_flow_factor;
            flow_acceleration
                + config.objective.slow_force(
                    velocity,
                    position,
                    target_cell_position,
                    flow_acceleration,
                ) * slow_factor
        } else {
            // TODO figure out why this logs sometimes. Commenting out to avoid spamming.
            // warn!(
            //     "Missing target_cell. This is okay if it's only for one frame. {:?}",
            //     target_cell
            // );
            Acceleration::ZERO
        }
    }
}
