use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use derive_more::{Add, AddAssign, Sub, SubAssign};
use std::ops::Mul;

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PhysicsMaterialType>()
            .register_type::<HashMap<PhysicsMaterialType, PhysicsMaterial>>()
            .register_type::<PhysicsMaterial>()
            .register_type::<PhysicsMaterials>()
            .add_systems(FixedUpdate, update.in_set(SystemStage::Apply));
    }
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
)]
pub struct Velocity(pub Vec2);
impl Velocity {
    pub const ZERO: Self = Velocity(Vec2::ZERO);
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
)]
pub struct Acceleration(pub Vec2);
impl Acceleration {
    pub const ZERO: Self = Acceleration(Vec2::ZERO);
}
impl Mul<f32> for Acceleration {
    type Output = Acceleration;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0.mul(rhs))
    }
}

/// Apply velocity changes.
pub fn update(
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        &mut Acceleration,
        &PhysicsMaterialType,
    )>,
    materials: Res<PhysicsMaterials>,
    grid_spec: Res<GridSpec>,
    obstacles: Res<Grid2<Obstacle>>,
) {
    for (mut transform, mut velocity, mut acceleration, material_type) in &mut query {
        let material = materials.get(material_type).unwrap();
        let prev_velocity = *velocity;

        velocity.0 += acceleration.0;
        let overflow = velocity.length_squared() / (material.max_velocity.powi(2)) * 0.1;
        velocity.0 = velocity.clamp_length_max(material.max_velocity);
        velocity.0 *= overflow.clamp(1.0, 10.0);
        velocity.0 = velocity.lerp(prev_velocity.0, material.velocity_smoothing);

        let position = transform.translation.xy();
        let new_position = position + velocity.0;
        if obstacles[obstacles.to_rowcol(new_position)] == Obstacle::Empty {
            transform.translation = new_position.extend(0.);
        } else {
            velocity.0 *= -0.5;
            transform.translation += velocity.0.extend(0.);
        }

        grid_spec
            .world2d_bounds_eps()
            .clamp3(&mut transform.translation);

        acceleration.0 = Vec2::ZERO;
    }
}

#[derive(Resource, Clone, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct PhysicsMaterials(HashMap<PhysicsMaterialType, PhysicsMaterial>);

#[derive(Component, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect, Debug)]
pub enum PhysicsMaterialType {
    #[default]
    Default,
    Zooid,
    SlowZooid,
    Plankton,
}
#[derive(Clone, Reflect, Debug)]
pub struct PhysicsMaterial {
    max_velocity: f32,
    velocity_smoothing: f32,
}
impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            max_velocity: 10.0,
            velocity_smoothing: 0.,
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct PhysicsBundle {
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub material: PhysicsMaterialType,
}
