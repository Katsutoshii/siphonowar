use std::f32::consts::PI;

use self::effects::{EffectCommands, EffectSize, FireworkSpec};

use super::{
    carry::{CarriedBy, Carrier, CarryEvent},
    neighbors::{AlliedNeighbors, EnemyNeighbors},
    DamageEvent, InteractionConfig, ObjectSpec,
};
use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

/// Plugin for running zooids simulation.
pub struct ObjectPlugin;
impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Object>().add_systems(
            FixedUpdate,
            (
                Object::update_acceleration.in_set(SystemStage::Compute),
                Object::update_objective.in_set(SystemStage::Compute),
                Object::death.in_set(SystemStage::Despawn),
                ObjectBackground::update.in_set(SystemStage::Compute),
            ),
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

#[derive(Clone)]
struct NearestNeighbor {
    pub distance_squared: f32,
    pub entity: Entity,
    pub object: Object,
    pub velocity: Velocity,
    pub carrier: Option<Carrier>,
    pub carried_by: Option<CarriedBy>,
}
trait NearestNeighborExtension {
    fn distance_squared(&self) -> f32;
}
impl NearestNeighborExtension for Option<NearestNeighbor> {
    fn distance_squared(&self) -> f32 {
        if let Some(nearest_neighbor) = self {
            nearest_neighbor.distance_squared
        } else {
            f32::INFINITY
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct UpdateAccelerationQueryData {
    entity: Entity,
    object: &'static Object,
    velocity: &'static Velocity,
    acceleration: &'static mut Acceleration,
    carrier: Option<&'static Carrier>,
    neighbors: &'static AlliedNeighbors,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct UpdateObjectiveQueryData {
    entity: Entity,
    object: &'static Object,
    objectives: &'static mut Objectives,
    carrier: Option<&'static Carrier>,
    health: &'static Health,
    neighbors: &'static EnemyNeighbors,
}

#[derive(QueryData)]
pub struct UpdateObjectiveNeighborQueryData {
    object: &'static Object,
    velocity: &'static Velocity,
    carrier: Option<&'static Carrier>,
    carried_by: Option<&'static CarriedBy>,
}

impl Object {
    pub fn update_acceleration(
        mut query: Query<UpdateAccelerationQueryData>,
        others: Query<(&Self, &Velocity)>,
        configs: Res<Configs>,
    ) {
        query.par_iter_mut().for_each(|mut object| {
            let mut seaparation_acceleration = Acceleration::ZERO;
            let mut alignment_acceleration = Acceleration::ZERO;
            let config = &configs.objects[object.object];
            for neighbor in object.neighbors.iter() {
                let (other_object, other_velocity) = others.get(neighbor.entity).unwrap();
                let interaction = &config.interactions[other_object];
                let radius_squared = config.neighbor_radius * config.neighbor_radius;

                // Don't apply neighbor forces when carrying items.
                if object.carrier.is_none() {
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
            if !object.neighbors.is_empty() {
                *object.acceleration += alignment_acceleration
                    * (1.0 / (object.neighbors.len() as f32))
                    + seaparation_acceleration;
            }
            let spin_amount = (config.idle_speed * 2. - object.velocity.length_squared()).max(0.);
            let turn_vector = Mat2::from_angle(PI / 8.) * object.velocity.0 * spin_amount;
            *object.acceleration += Acceleration(turn_vector);
        });
    }

    pub fn update_objective(
        mut query: Query<UpdateObjectiveQueryData>,
        others: Query<UpdateObjectiveNeighborQueryData>,
        configs: Res<Configs>,
        mut damage_events: EventWriter<DamageEvent>,
        mut carry_events: EventWriter<CarryEvent>,
    ) {
        for mut object in &mut query {
            let config = &configs.objects[object.object];
            let mut nearest_neighbor: Option<NearestNeighbor> = None;
            for neighbor in object.neighbors.iter() {
                let other = others.get(neighbor.entity).unwrap();

                if neighbor.distance_squared < nearest_neighbor.distance_squared() {
                    nearest_neighbor = Some(NearestNeighbor {
                        distance_squared: neighbor.distance_squared,
                        entity: neighbor.entity,
                        velocity: *other.velocity,
                        object: *other.object,
                        carrier: other.carrier.copied(),
                        carried_by: other.carried_by.cloned(),
                    });
                }

                // Food specific behavior.
                let radius_squared = config.neighbor_radius * config.neighbor_radius;
                if *object.object == Object::Food
                    && neighbor.object == Object::Head
                    && neighbor.distance_squared < radius_squared * 0.1
                {
                    damage_events.send(DamageEvent {
                        damager: neighbor.entity,
                        damaged: object.entity,
                        amount: 1,
                        velocity: Velocity::ZERO,
                    });
                }
            }
            if let Some(neighbor) = nearest_neighbor {
                // An object should only attack a neighbor if that neighbor is not being carried.
                if object.object.can_attack()
                    && neighbor.object.can_be_attacked()
                    && object.carrier.is_none()
                    && neighbor.carried_by.is_none()
                {
                    object.objectives.start_attacking(neighbor.entity)
                }
                let interaction = &config.interactions[&neighbor.object];
                if config.is_colliding(neighbor.distance_squared) {
                    // If we can carry
                    if object.object.can_be_carried()
                        && neighbor.object.can_carry()
                        && neighbor.carrier.is_none()
                    {
                        carry_events.send(CarryEvent {
                            carrier: neighbor.entity,
                            carried: object.entity,
                        });
                    }
                    // If we can be damaged this frame.
                    else if interaction.damage_amount > 0
                        && config.is_damage_velocity(neighbor.velocity.length_squared())
                        && object.health.damageable()
                    {
                        damage_events.send(DamageEvent {
                            damager: neighbor.entity,
                            damaged: object.entity,
                            amount: interaction.damage_amount,
                            velocity: neighbor.velocity,
                        });
                    }
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
        mut objects: Query<(Entity, &Self, &GridEntity, &Health, &Transform, &Team)>,
        mut commands: Commands,
        mut object_commands: ObjectCommands,
        mut effect_commands: EffectCommands,
        mut grid: ResMut<Grid2<EntitySet>>,
    ) {
        for (entity, object, grid_entity, health, transform, team) in &mut objects {
            if health.health <= 0 {
                grid.remove(entity, grid_entity);
                commands.entity(entity).despawn_recursive();
                effect_commands.make_fireworks(FireworkSpec {
                    size: EffectSize::Medium,
                    transform: *transform,
                    team: *team,
                });
                if object == &Object::Plankton {
                    object_commands.spawn(ObjectSpec {
                        object: Object::Food,
                        position: transform.translation.xy(),
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
        mut query: Query<(&mut Transform, &Parent), With<Self>>,
        parent_velocities: Query<&Velocity, With<Children>>,
    ) {
        for (mut transform, parent) in &mut query {
            let parent_velocity = parent_velocities
                .get(parent.get())
                .expect("Invalid parent.");
            transform.translation = -0.1 * parent_velocity.extend(0.);
        }
    }
}
