use std::f32::consts::PI;

use super::{
    carry::{CarriedBy, CarryEvent},
    neighbors::{AlliedNeighbors, CollidingNeighbors, EnemyNeighbors},
    InteractionConfig, ObjectSpec,
};
use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
use rand::Rng;
use sipho_vfx::fireworks::EffectCommands;
/// Plugin for running zooids simulation.
pub struct ObjectPlugin;
impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Object>().add_systems(
            FixedUpdate,
            (
                (
                    Object::update_acceleration,
                    Object::update_objective,
                    Object::update_collisions,
                    ObjectBackground::update,
                )
                    .in_set(SystemStage::Compute),
                Object::death.in_set(SystemStage::Despawn),
            )
                .in_set(GameStateSet::Running),
        );
    }
}

/// Entities that can interact with each other.
#[derive(Component, Reflect, Default, Copy, Clone, PartialEq, Eq, Hash, Debug, clap::ValueEnum)]
#[reflect(Component)]
pub enum Object {
    #[default]
    Worker,
    Head,
    Plankton,
    Food,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct UpdateAccelerationQueryData {
    entity: Entity,
    object: &'static Object,
    velocity: &'static Velocity,
    acceleration: &'static mut Acceleration,
    parent: Option<&'static Parent>,
    objectives: &'static Objectives,
    neighbors: &'static AlliedNeighbors,
    enemy_neighbors: &'static EnemyNeighbors,
    carried_by: Option<&'static CarriedBy>,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct UpdateObjectiveQueryData {
    entity: Entity,
    object: &'static Object,
    objectives: &'static mut Objectives,
    parent: Option<&'static Parent>,
    health: &'static Health,
    enemy_neighbors: &'static EnemyNeighbors,
    allied_neighbors: &'static AlliedNeighbors,
}

#[derive(QueryData)]
pub struct UpdateObjectiveNeighborQueryData {
    object: &'static Object,
    velocity: &'static Velocity,
    parent: Option<&'static Parent>,
    carried_by: Option<&'static CarriedBy>,
}

impl Object {
    pub fn update_acceleration(
        mut query: Query<UpdateAccelerationQueryData>,
        others: Query<(&Self, &Velocity)>,
        configs: Res<ObjectConfigs>,
    ) {
        query.par_iter_mut().for_each(|mut object| {
            let mut separation_acceleration = Acceleration::ZERO;
            let mut alignment_acceleration = Acceleration::ZERO;
            let config = configs.get(object.object).unwrap();

            for neighbor in object.neighbors.iter() {
                let (other_object, other_velocity) = others.get(neighbor.entity).unwrap();
                let interaction = &config.interactions[other_object];
                let radius_squared = config.neighbor_radius * config.neighbor_radius;

                // Don't apply neighbor forces when carrying items.
                if object.parent.is_none() {
                    let separation_radius_factor = 2.;
                    separation_acceleration += Self::separation_acceleration(
                        -neighbor.delta,
                        neighbor.distance_squared,
                        *object.velocity,
                        interaction,
                        separation_radius_factor,
                    );
                    alignment_acceleration += Self::alignment_acceleration(
                        neighbor.distance_squared,
                        radius_squared,
                        *object.velocity,
                        *other_velocity,
                        interaction,
                    );
                }
            }
            for neighbor in object.enemy_neighbors.iter() {
                let (other_object, _other_velocity) = others.get(neighbor.entity).unwrap();
                let separation_radius_factor = 3.;
                separation_acceleration += Self::separation_acceleration(
                    -neighbor.delta,
                    neighbor.distance_squared,
                    *object.velocity,
                    &config.interactions[other_object],
                    separation_radius_factor,
                );
            }

            if !object.neighbors.is_empty() {
                *object.acceleration +=
                    alignment_acceleration * (1.0 / (object.neighbors.len() as f32));
            }
            *object.acceleration += separation_acceleration;

            // When idle, slow down.
            if *object.objectives.last() == Objective::None && object.carried_by.is_none() {
                let idle_slow_threshold = config.idle_speed;
                let velocity_squared = object.velocity.length_squared();
                if velocity_squared > 0. {
                    let slow_magnitude =
                        0.3 * (velocity_squared - idle_slow_threshold).max(0.) / velocity_squared;
                    *object.acceleration += Acceleration(-object.velocity.0 * slow_magnitude)
                }
            }

            // When moving slow, spin around to create some extra movement.
            let random_factor = rand::thread_rng().gen_range(0.8..1.0);
            let spin_amount = (config.idle_speed * 2. - object.velocity.length_squared()).max(0.0)
                * (random_factor)
                * 2.;
            let turn_vector =
                Mat2::from_angle(PI * random_factor / 8.) * object.velocity.0 * spin_amount;
            *object.acceleration += Acceleration(turn_vector);
        });
    }

    pub fn update_objective(
        mut query: Query<UpdateObjectiveQueryData>,
        others: Query<UpdateObjectiveNeighborQueryData>,
    ) {
        for mut object in &mut query {
            let nearest = object.enemy_neighbors.iter().next();

            if let Some(neighbor) = nearest {
                let other = others.get(neighbor.entity).unwrap();
                // An object should only attack a neighbor if that neighbor is not being carried.
                if object.object.can_attack()
                    && neighbor.object.can_be_attacked()
                    && object.parent.is_none()
                    && other.carried_by.is_none()
                {
                    // If already attacking an entity but we are now closer to different entity, attack the new closest
                    // entity.
                    if let Objective::AttackEntity(entity) =
                        object.objectives.bypass_change_detection().last_mut()
                    {
                        *entity = neighbor.entity;
                    } else {
                        object
                            .objectives
                            .push(Objective::AttackEntity(neighbor.entity));
                    }
                }
            }
        }
    }

    pub fn update_collisions(
        mut objects: Query<(Entity, &Object, &CollidingNeighbors, Option<&Parent>)>,
        // others: Query<(&Object, Option<&DashAttacker>, &Velocity)>,
        // configs: Res<ObjectConfigs>,
        // mut damage_events: EventWriter<DamageEvent>,
        mut carry_events: EventWriter<CarryEvent>,
    ) {
        for (entity, object, collisions, parent) in objects.iter_mut() {
            // let config = configs.get(object).unwrap();
            for neighbor in collisions.iter() {
                // let (other_object, other_attacker, other_velocity) =
                //     others.get(neighbor.entity).unwrap();

                if object.can_carry() && neighbor.object.can_be_carried() && parent.is_none() {
                    carry_events.send(CarryEvent {
                        carrier: entity,
                        carried: neighbor.entity,
                    });
                }
            }
        }
    }

    /// Returns true if an object can attack.
    pub fn can_attack(self) -> bool {
        match self {
            Self::Worker => true,
            Self::Food | Self::Head | Self::Plankton => false,
        }
    }

    /// Returns true if an object can be attacked.
    pub fn can_be_attacked(self) -> bool {
        match self {
            Self::Worker | Self::Head | Self::Plankton => true,
            Self::Food => true,
        }
    }

    /// Returns true if an object can carry.
    pub fn can_carry(self) -> bool {
        match self {
            Self::Worker => true,
            Self::Food | Self::Head | Self::Plankton => false,
        }
    }

    /// Returns true if an object can carry.
    pub fn can_be_carried(self) -> bool {
        match self {
            Self::Food => true,
            Self::Worker | Self::Head | Self::Plankton => false,
        }
    }

    /// System for objects dying.
    pub fn death(
        mut objects: Query<(Entity, &Self, &GridEntity, &Health, &GlobalTransform, &Team)>,
        mut object_commands: ObjectCommands,
        mut effect_commands: EffectCommands,
    ) {
        for (entity, object, grid_entity, health, &transform, team) in &mut objects {
            if health.health <= 0 {
                object_commands.despawn(entity, *grid_entity);
                effect_commands.make_fireworks(FireworkSpec {
                    size: VfxSize::Medium,
                    transform: transform.into(),
                    team: *team,
                });
                if object == &Object::Plankton {
                    object_commands.spawn(ObjectSpec {
                        object: Object::Food,
                        position: transform.translation().xy(),
                        ..default()
                    })
                }
            }
        }
    }

    /// Compute acceleration from separation.
    /// The direction is towards self away from each nearby bird.
    /// The magnitude is computed by
    /// $ magnitude = sep * (-x^2 / r^2 + 1)$
    fn separation_acceleration(
        position_delta: Vec2,
        distance_squared: f32,
        velocity: Velocity,
        interaction: &InteractionConfig,
        radius_factor: f32,
    ) -> Acceleration {
        let radius = radius_factor * interaction.separation_radius;
        let radius_squared = radius * radius;

        let slow_force = interaction.slow_factor
            * if distance_squared < radius_squared {
                Vec2::ZERO
            } else {
                -1.0 * velocity.0
            };

        let magnitude =
            interaction.separation_acceleration * (-distance_squared / (radius_squared) + 1.);
        Acceleration(
            position_delta.normalize_or_zero()
                * magnitude.clamp(
                    -interaction.cohesion_acceleration,
                    interaction.separation_acceleration,
                )
                + slow_force,
        )
    }

    /// Alignment acceleration.
    /// Compute the difference between this object's velocity and the other object's velocity.
    fn alignment_acceleration(
        distance_squared: f32,
        radius_squared: f32,
        velocity: Velocity,
        other_velocity: Velocity,
        config: &InteractionConfig,
    ) -> Acceleration {
        let magnitude = (radius_squared - distance_squared) / radius_squared;
        Acceleration((other_velocity.0 - velocity.0) * config.alignment_factor * magnitude)
    }
}

#[derive(Component, Default)]
pub struct ObjectBackground;
impl ObjectBackground {
    pub fn update(
        mut query: Query<(Entity, &mut Transform, Option<&Parent>), With<Self>>,
        parent_velocities: Query<&Velocity, With<Children>>,
        mut commands: Commands,
    ) {
        for (entity, mut transform, parent) in &mut query {
            if let Some(parent) = parent {
                if let Ok(parent_velocity) = parent_velocities.get(parent.get()) {
                    transform.translation = -0.1 * parent_velocity.extend(0.);
                }
            } else {
                info!("No parent, despawn background! {:?}", entity);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
