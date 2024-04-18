use std::f32::consts::PI;
use strum_macros::IntoStaticStr;

use super::{
    carry::CarriedBy,
    neighbors::{AlliedNeighbors, CollidingNeighbors, EnemyNeighbors},
    path_to_head::PathToHeadFollower,
    InteractionConfig, ObjectSpec,
};
use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
use rand::Rng;

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
                Object::death.in_set(SystemStage::Death),
            )
                .in_set(GameStateSet::Running),
        );
    }
}

/// Entities that can interact with each other.
#[derive(
    Component,
    Reflect,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    clap::ValueEnum,
    IntoStaticStr,
)]
#[reflect(Component)]
pub enum Object {
    #[default]
    Worker,
    Head,
    Plankton,
    Food,
    Shocker,
    Armor,
}
impl Object {
    pub const ALL: [Object; 6] = [
        Self::Worker,
        Self::Head,
        Self::Plankton,
        Self::Food,
        Self::Shocker,
        Self::Armor,
    ];

    /// Returns true if an object can attack.
    pub fn can_attack(self) -> bool {
        matches!(self, Self::Worker | Self::Shocker)
    }

    /// Returns true if an object can be attacked.
    pub fn can_be_attacked(self) -> bool {
        true
    }

    /// Returns true if an object can carry.
    pub fn can_carry(self) -> bool {
        matches!(self, Self::Worker)
    }

    /// Returns true if an object can carry.
    pub fn can_be_carried(self) -> bool {
        matches!(self, Self::Food)
    }
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
    carried_by: &'static CarriedBy,
    attached_to: &'static AttachedTo,
    path_follower: Option<&'static PathToHeadFollower>,
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
    attached_to: &'static AttachedTo,
}

#[derive(QueryData)]
pub struct UpdateObjectiveNeighborQueryData {
    object: &'static Object,
    velocity: &'static Velocity,
    parent: Option<&'static Parent>,
    carried_by: &'static CarriedBy,
    path_follower: Option<&'static PathToHeadFollower>,
}

impl Object {
    pub fn zindex(self) -> f32 {
        match self {
            Self::Worker => zindex::ZOOIDS_MIN,
            Self::Head => zindex::ZOOID_HEAD,
            Self::Food => zindex::FOOD,
            Self::Plankton => zindex::PLANKTON,
            Self::Shocker => zindex::ZOOIDS_MIN,
            Self::Armor => zindex::ZOOIDS_MIN,
        }
    }
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

            // For objects being piped through an organism, skip enemy forces.
            if object.path_follower.is_none() || object.path_follower.unwrap().target.is_none() {
                for neighbor in object.enemy_neighbors.iter() {
                    let (other_object, _other_velocity) = others.get(neighbor.entity).unwrap();
                    let separation_radius_factor = 3.;
                    separation_acceleration += Self::separation_acceleration(
                        -neighbor.delta,
                        neighbor.distance_squared,
                        &config.interactions[other_object],
                        separation_radius_factor,
                    );
                }
            }

            if !object.neighbors.is_empty() {
                *object.acceleration +=
                    alignment_acceleration * (1.0 / (object.neighbors.len() as f32));
            }
            *object.acceleration += separation_acceleration;

            // When idle, slow down.
            if *object.objectives.last() == Objective::Idle
                && object.carried_by.is_empty()
                && object.attached_to.is_empty()
            {
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
            if let Some(neighbor) = object.enemy_neighbors.first() {
                let other = others.get(neighbor.entity).unwrap();
                // An object should only attack a neighbor if that neighbor is not being carried.
                if object.object.can_attack()
                    && object.attached_to.len() < 2
                    && neighbor.object.can_be_attacked()
                    && object.parent.is_none()
                    && other.carried_by.is_empty()
                    && (other.path_follower.is_none()
                        || other.path_follower.unwrap().target.is_none())
                {
                    // If already attacking an entity but we are now closer to different entity, attack the new closest
                    // entity.
                    match object.objectives.bypass_change_detection().last_mut() {
                        Objective::AttackEntity(entity) => {
                            *entity = neighbor.entity;
                        }
                        Objective::AttackFollowEntity(_) | Objective::Idle => {
                            object
                                .objectives
                                .push(Objective::AttackEntity(neighbor.entity));
                        }
                        Objective::FollowEntity(_) => {}
                        Objective::Stunned(timer) => {
                            if timer.finished() {
                                object.objectives.pop();
                                object.objectives.push(Objective::Idle);
                            }
                        }
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
            Option<&mut PathToHeadFollower>,
            Option<&Parent>,
        )>,
        // mut carry_events: EventWriter<CarryEvent>,
        path_to_head: Query<&PathToHead>,
    ) {
        for (_entity, object, collisions, mut path_follower, parent) in objects.iter_mut() {
            for neighbor in collisions.iter() {
                if let Some(ref mut path_follower) = path_follower {
                    // If already following a path, don't go collide with others.
                    if let Some(target) = path_follower.target {
                        if target != neighbor.entity {
                            continue;
                        }
                    }
                    if let Ok(path) = path_to_head.get(neighbor.entity) {
                        path_follower.target = if path.next.is_some() {
                            path.next
                        } else {
                            path.head
                        };
                    }
                } else if object.can_carry() && neighbor.object.can_be_carried() && parent.is_none()
                {
                    // carry_events.send(CarryEvent {
                    //     carrier: entity,
                    //     carried: neighbor.entity,
                    // });
                }
            }
        }
    }

    /// System for objects dying.
    pub fn death(
        mut objects: Query<(Entity, &Self, &Health, &GlobalTransform, &Team)>,
        mut object_commands: ObjectCommands,
        mut firework_events: EventWriter<FireworkSpec>,
    ) {
        for (entity, object, health, &transform, team) in &mut objects {
            if health.health <= 0 {
                object_commands.despawn(entity);
                firework_events.send(FireworkSpec {
                    size: VfxSize::Medium,
                    position: transform.translation(),
                    color: (*team).into(),
                });
                if object == &Object::Plankton {
                    object_commands.spawn(ObjectSpec {
                        object: Object::Food,
                        position: transform.translation().xy(),
                        ..default()
                    });
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
        interaction: &InteractionConfig,
        radius_factor: f32,
    ) -> Acceleration {
        let radius = radius_factor * interaction.separation_radius;
        let radius_squared = radius * radius;
        let magnitude =
            interaction.separation_acceleration * (-distance_squared / (radius_squared) + 1.);
        Acceleration(
            position_delta.normalize_or_zero()
                * magnitude.clamp(
                    -interaction.cohesion_acceleration,
                    interaction.separation_acceleration,
                ),
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
        parents: Query<(&GlobalTransform, &Velocity), With<Children>>,
    ) {
        for (entity, mut transform, parent) in &mut query {
            if let Some(parent) = parent {
                if let Ok((parent_transform, parent_velocity)) = parents.get(parent.get()) {
                    let offset = -0.1 * parent_velocity.extend(0.);
                    let inverse = parent_transform.compute_transform().rotation.inverse();
                    let result = inverse.mul_vec3(offset);
                    transform.translation = result;
                }
            } else {
                info!("No parent, despawn background! {:?}", entity);
            }
        }
    }
}
