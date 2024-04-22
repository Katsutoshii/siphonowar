use std::f32::consts::PI;
use strum_macros::IntoStaticStr;

use super::{
    carry::CarriedBy,
    neighbors::{AlliedNeighbors, EnemyCollisions, EnemyNeighbors},
    path_to_head::PathToHeadFollower,
    InteractionConfig,
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
            ((
                Object::update_force,
                Object::update_collisions,
                ObjectBackground::update,
            )
                .in_set(FixedUpdateStage::AccumulateForces),)
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
pub struct UpdateforceQueryData {
    entity: Entity,
    object: &'static Object,
    velocity: &'static Velocity,
    force: &'static mut Force,
    parent: Option<&'static Parent>,
    objectives: &'static Objectives,
    neighbors: &'static AlliedNeighbors,
    enemy_neighbors: &'static EnemyNeighbors,
    carried_by: &'static CarriedBy,
    attached_to: &'static AttachedTo,
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
    pub fn update_force(
        mut query: Query<UpdateforceQueryData>,
        others: Query<(&Self, &Velocity)>,
        configs: Res<ObjectConfigs>,
    ) {
        query.par_iter_mut().for_each(|mut object| {
            let mut separation_force = Force::ZERO;
            let mut alignment_force = Force::ZERO;
            let config = configs.get(object.object).unwrap();

            for neighbor in object.neighbors.iter() {
                let (other_object, other_velocity) = others.get(neighbor.entity).unwrap();
                let interaction = &config.interactions[other_object];
                let radius_squared = config.neighbor_radius * config.neighbor_radius;

                // Don't apply neighbor forces when carrying items.
                if object.parent.is_none() {
                    let separation_radius_factor = 2.;
                    separation_force += Self::separation_force(
                        -neighbor.delta,
                        neighbor.distance_squared,
                        interaction,
                        separation_radius_factor,
                    );
                    alignment_force += Self::alignment_force(
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
                    separation_force += Self::separation_force(
                        -neighbor.delta,
                        neighbor.distance_squared,
                        &config.interactions[other_object],
                        separation_radius_factor,
                    );
                }
            }

            if !object.neighbors.is_empty() {
                *object.force += alignment_force * (1.0 / (object.neighbors.len() as f32));
            }
            *object.force += separation_force;

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
                    *object.force += Force(-object.velocity.0 * slow_magnitude)
                }
            }

            // When moving slow, spin around to create some extra movement.
            let random_factor = rand::thread_rng().gen_range(0.8..1.0);
            let spin_amount = (config.idle_speed * 2. - object.velocity.length_squared()).max(0.0)
                * (random_factor)
                * 2.;
            let turn_vector =
                Mat2::from_angle(PI * random_factor / 8.) * object.velocity.0 * spin_amount;
            *object.force += Force(turn_vector);
        });
    }

    pub fn update_collisions(
        mut objects: Query<(
            Entity,
            &Object,
            &EnemyCollisions,
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

    /// Compute force from separation.
    /// The direction is towards self away from each nearby bird.
    /// The magnitude is computed by
    /// $ magnitude = sep * (-x^2 / r^2 + 1)$
    fn separation_force(
        position_delta: Vec2,
        distance_squared: f32,
        interaction: &InteractionConfig,
        radius_factor: f32,
    ) -> Force {
        let radius = radius_factor * interaction.separation_radius;
        let radius_squared = radius * radius;
        let magnitude = interaction.separation_force * (-distance_squared / (radius_squared) + 1.);
        Force(
            position_delta.normalize_or_zero()
                * magnitude.clamp(-interaction.cohesion_force, interaction.separation_force),
        )
    }

    /// Alignment force.
    /// Compute the difference between this object's velocity and the other object's velocity.
    fn alignment_force(
        distance_squared: f32,
        radius_squared: f32,
        velocity: Velocity,
        other_velocity: Velocity,
        config: &InteractionConfig,
    ) -> Force {
        let magnitude = (radius_squared - distance_squared) / radius_squared;
        Force((other_velocity.0 - velocity.0) * config.alignment_factor * magnitude)
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
