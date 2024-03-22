/// Objectives define what an object is trying to do.
/// We maintain a stack of objectives for each object.
/// Each frame, we check the current object and try to resolve it to the corresponding behavior components.
use crate::prelude::*;
use bevy::ecs::query::QueryData;

pub mod config;
pub mod dash_attacker;
pub mod debug;
pub mod navigator;

use self::{dash_attacker::DashAttacker, navigator::Navigator};

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
                (Objectives::update_components, ObjectiveDebugger::update)
                    .chain()
                    .in_set(SystemStage::PostApply),
            )
            .add_plugins((
                navigator::NavigatorPlugin,
                dash_attacker::DashAttackerPlugin,
            ));
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ObjectivesQueryData {
    entity: Entity,
    navigator: Option<&'static mut Navigator>,
    dash_attacker: Option<&'static mut DashAttacker>,
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
    AttackEntity(Entity),
}
impl Objective {
    pub fn try_update_components(
        &self,
        components: &mut ObjectivesQueryDataItem,
        transforms: &Query<&GlobalTransform>,
        commands: &mut Commands,
    ) -> bool {
        match self {
            Self::None => {
                if components.dash_attacker.is_some() {
                    commands.entity(components.entity).remove::<DashAttacker>();
                }
                if components.navigator.is_some() {
                    commands.entity(components.entity).remove::<Navigator>();
                }
                true
            }
            Self::FollowEntity(entity) => {
                if let Ok(transform) = transforms.get(*entity) {
                    if components.dash_attacker.is_some() {
                        commands.entity(components.entity).remove::<DashAttacker>();
                    }
                    if let Some(ref mut navigator) = components.navigator {
                        navigator.target = transform.translation().xy();
                    } else {
                        commands.entity(components.entity).insert(Navigator {
                            target: transform.translation().xy(),
                            slow_factor: 1.0,
                        });
                    }
                    true
                } else {
                    false
                }
            }
            Self::AttackEntity(entity) => {
                if let Ok(transform) = transforms.get(*entity) {
                    if let Some(ref mut navigator) = components.navigator {
                        navigator.target = transform.translation().xy();
                    } else {
                        commands.entity(components.entity).insert(Navigator {
                            target: transform.translation().xy(),
                            slow_factor: 1.0,
                        });
                    }

                    if let Some(ref mut dash_attacker) = components.dash_attacker {
                        dash_attacker.target = transform.translation().xy();
                    } else {
                        commands.entity(components.entity).insert(DashAttacker {
                            target: transform.translation().xy(),
                            ..default()
                        });
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Given an objective, get the next one (if there should be a next one, else None).
    pub fn try_attacking(&self, entity: Entity) -> Option<Self> {
        match self {
            Self::None | Self::FollowEntity(_) => Some(Self::AttackEntity(entity)),
            Self::AttackEntity { .. } => None,
        }
    }

    /// If this objective is following an entity, return that Entity.
    pub fn get_followed_entity(&self) -> Option<Entity> {
        match self {
            Self::AttackEntity(entity) | Self::FollowEntity(entity) => Some(*entity),
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
    pub fn update_components(
        mut query: Query<(&mut Objectives, ObjectivesQueryData)>,
        transforms: Query<&GlobalTransform>,
        mut commands: Commands,
    ) {
        for (mut objectives, mut components) in query.iter_mut() {
            loop {
                if objectives.last().try_update_components(
                    &mut components,
                    &transforms,
                    &mut commands,
                ) {
                    break;
                } else {
                    objectives.pop();
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

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
}
