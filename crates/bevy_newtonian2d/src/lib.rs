/// Super basic Newtonian physics simulation for Bevy.
use bevy::prelude::*;
use derive_more::{Add, AddAssign, Sub, SubAssign};
use std::ops::Mul;

/// Plugin for basic 2d physics.
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PhysicsMaterial>()
            .register_type::<Mass>()
            .register_type::<Velocity>()
            .register_type::<Force>()
            .init_state::<PhysicsSimulationState>()
            .add_systems(Update, update.in_set(PhysicsSystem::UpdateTransform))
            .configure_sets(
                FixedUpdate,
                (PhysicsSystem::AccumulateForces, PhysicsSystem::ApplyForces).chain(),
            )
            .configure_sets(Update, PhysicsSystem::UpdateTransform)
            .add_systems(
                FixedUpdate,
                (fixed_update_children, fixed_update)
                    .chain()
                    .in_set(PhysicsSystem::ApplyForces)
                    .run_if(in_state(PhysicsSimulationState::Running)),
            );
    }
}

/// Tracks Mass per entity.
#[derive(
    Component,
    Debug,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    PartialEq,
    Reflect,
)]
pub struct Mass(pub f32);
impl Default for Mass {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Tracks position per entity.
#[derive(
    Component,
    Debug,
    Default,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    PartialEq,
    Reflect,
)]
pub struct Position(pub Vec2);
impl Position {
    pub const ZERO: Self = Self(Vec2::ZERO);
}

/// Tracks velocity per entity.
#[derive(
    Component,
    Debug,
    Default,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    PartialEq,
    Reflect,
)]
pub struct Velocity(pub Vec2);
impl Velocity {
    pub const ZERO: Self = Self(Vec2::ZERO);
}
impl Mul<f32> for Velocity {
    type Output = Velocity;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0.mul(rhs))
    }
}

/// Tracks new velocity per entity, which can be used for double-buffering
/// velocity updates.
#[derive(
    Component,
    Debug,
    Default,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    PartialEq,
    Reflect,
)]
pub struct Force(pub Vec2);
impl Force {
    pub const ZERO: Self = Self(Vec2::ZERO);
}
impl Mul<f32> for Force {
    type Output = Force;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0.mul(rhs))
    }
}

// Propagate position to transform.
pub fn update(mut query: Query<(&Position, &mut Transform), Without<Parent>>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

/// Apply velocity changes.
pub fn fixed_update(
    mut query: Query<
        (
            &mut Position,
            &mut Velocity,
            &mut Force,
            &Mass,
            &PhysicsMaterial,
        ),
        Without<Parent>,
    >,
) {
    for (mut position, mut velocity, mut force, mass, material) in &mut query {
        let prev_velocity = *velocity;

        velocity.0 += force.0 / (mass.0).max(0.1);
        let overflow = velocity.length_squared() / (material.max_velocity.powi(2)) * 0.1;
        velocity.0 = velocity.clamp_length_max(material.max_velocity);
        velocity.0 *= overflow.clamp(1.0, 10.0);
        velocity.0 = velocity.lerp(prev_velocity.0, material.velocity_smoothing);

        position.0 += velocity.0;

        *force = Force::ZERO;
    }
}

// For simulated objects that are parented, apply child forces on the parent.
// Update child velocity so it can be read elsewhere.
pub fn fixed_update_children(
    mut parents_query: Query<(&Velocity, &mut Force, &Children), Without<Parent>>,
    mut children_query: Query<(&mut Position, &mut Velocity, &mut Force), With<Parent>>,
) {
    for (velocity, mut force, children) in parents_query.iter_mut() {
        // Sum all child forces.
        let mut children_force = Force::ZERO;
        let mut num_children = 0;
        for &child in children.iter() {
            if let Ok((mut child_position, mut child_velocity, mut child_force)) =
                children_query.get_mut(child)
            {
                num_children += 1;
                children_force += *child_force;
                *child_velocity = *velocity;
                *child_force = Force::ZERO;

                child_position.0 += velocity.0;
            }
        }
        if num_children > 0 {
            *force += children_force * (num_children as f32).recip();
        }
    }
}

#[derive(Clone, Reflect, Component, Debug)]
#[reflect(Component)]
pub struct PhysicsMaterial {
    max_velocity: f32,
    velocity_smoothing: f32,
}
impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            max_velocity: 10.0,
            velocity_smoothing: 0.0,
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct PhysicsBundle {
    pub position: Position,
    pub mass: Mass,
    pub velocity: Velocity,
    pub force: Force,
    pub material: PhysicsMaterial,
}

/// Set enum for the systems relating to transform propagation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PhysicsSystem {
    /// Propagates changes in transform to children's [`GlobalTransform`]
    AccumulateForces,
    ApplyForces,
    UpdateTransform,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSimulationState {
    /// Game is running.
    #[default]
    Running,
    /// Game is paused.
    Paused,
}
