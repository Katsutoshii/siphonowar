use std::f32::consts::PI;

use super::{
    carry::{CarriedBy, CarryEvent},
    neighbors::{AlliedNeighbors, CollidingNeighbors, EnemyNeighbors},
    DamageEvent, InteractionConfig, ObjectSpec,
};
use crate::{
    objectives::{dash_attacker::DashAttackerState, DashAttacker},
    prelude::*,
};
use bevy::{ecs::query::QueryData, prelude::*};
use rand::random;
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
    neighbors: &'static EnemyNeighbors,
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
            let mut seaparation_acceleration = Acceleration::ZERO;
            let mut alignment_acceleration = Acceleration::ZERO;
            let config = configs.get(object.object).unwrap();

            for neighbor in object.neighbors.iter() {
                let (other_object, other_velocity) = others.get(neighbor.entity).unwrap();
                let interaction = &config.interactions[other_object];
                let radius_squared = config.neighbor_radius * config.neighbor_radius;

                // Don't apply neighbor forces when carrying items.
                if object.parent.is_none() {
                    seaparation_acceleration += Self::separation_acceleration(
                        -neighbor.delta,
                        neighbor.distance_squared,
                        *object.velocity,
                        interaction,
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
                seaparation_acceleration += Self::separation_acceleration(
                    -neighbor.delta,
                    neighbor.distance_squared,
                    *object.velocity,
                    &config.interactions[others.get(neighbor.entity).unwrap().0],
                ) * 0.5;
            }

            if !object.neighbors.is_empty() {
                *object.acceleration += alignment_acceleration
                    * (1.0 / (object.neighbors.len() as f32))
                    + seaparation_acceleration;
            }

            // When idle, slow down.
            if *object.objectives.last() == Objective::None && object.carried_by.is_none() {
                let idle_slow_threshold = config.idle_speed;
                let velocity_squared = object.velocity.length_squared();
                if velocity_squared > 0. {
                    let slow_magnitude =
                        0.5 * (velocity_squared - idle_slow_threshold).max(0.) / velocity_squared;
                    *object.acceleration += Acceleration(-object.velocity.0 * slow_magnitude)
                }
            }

            // When moving slow, spin around to create some extra movement.
            let spin_amount = (config.idle_speed * 2. - object.velocity.length_squared()).max(0.)
                * random::<f32>()
                * 10.;
            let turn_vector = Mat2::from_angle(PI / 8.) * object.velocity.0 * spin_amount;
            *object.acceleration += Acceleration(turn_vector);
        });
    }

    pub fn update_objective(
        mut query: Query<UpdateObjectiveQueryData>,
        others: Query<UpdateObjectiveNeighborQueryData>,
    ) {
        for mut object in &mut query {
            let nearest = object
                .neighbors
                .iter()
                .min_by_key(|neighbor| bevy::utils::FloatOrd(neighbor.distance_squared));

            if let Some(neighbor) = nearest {
                let other = others.get(neighbor.entity).unwrap();
                // An object should only attack a neighbor if that neighbor is not being carried.
                if object.object.can_attack()
                    && neighbor.object.can_be_attacked()
                    && object.parent.is_none()
                    && other.carried_by.is_none()
                {
                    if let Some(objective) = object.objectives.last().try_attacking(neighbor.entity)
                    {
                        object.objectives.push(objective);
                    }
                }
            }
        }
    }

    pub fn update_collisions(
        mut objects: Query<(
            Entity,
            &Object,
            &CollidingNeighbors,
            &Health,
            Option<&Parent>,
        )>,
        others: Query<(&Object, Option<&DashAttacker>, &Velocity)>,
        configs: Res<ObjectConfigs>,
        mut damage_events: EventWriter<DamageEvent>,
        mut carry_events: EventWriter<CarryEvent>,
    ) {
        for (entity, object, collisions, health, parent) in objects.iter_mut() {
            let config = configs.get(object).unwrap();
            for neighbor in collisions.iter() {
                let (other_object, other_attacker, other_velocity) =
                    others.get(neighbor.entity).unwrap();
                let interaction = config.interactions.get(other_object).unwrap();

                if let Some(attacker) = other_attacker {
                    if attacker.state == DashAttackerState::Attacking {
                        damage_events.send(DamageEvent {
                            damager: neighbor.entity,
                            damaged: entity,
                            amount: if health.damageable() {
                                interaction.damage_amount
                            } else {
                                0
                            },
                            velocity: *other_velocity,
                        });
                    }
                }

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
    ) -> Acceleration {
        let radius = interaction.separation_radius;
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
