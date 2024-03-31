use std::slice::Iter;

/// Objectives define what an object is trying to do.
/// We maintain a stack of objectives for each object.
/// Each frame, we check the current object and try to resolve it to the corresponding behavior components.
use crate::prelude::*;
use bevy::ecs::query::QueryData;

use super::{dash_attacker::DashAttacker, navigator::Navigator};

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
    /// Entity follows the transform of another entity.
    FollowEntity(Entity),
    /// Entity follows the transform of another entity, or attacks
    AttackFollowEntity(Entity),
    /// Attack Entity
    AttackEntity(Entity),
}
impl Objective {
    /// When this objective is added, remove existing components.
    pub fn try_add_components(
        &self,
        components: &mut ObjectivesQueryDataItem,
        targets: &Query<(&GlobalTransform, &Velocity, Option<&CarriedBy>)>,
        commands: &mut Commands,
    ) -> Result<(), Error> {
        let mut commands = commands.entity(components.entity);
        commands.remove::<(DashAttacker, Navigator)>();
        match self {
            Self::None => {}
            Self::FollowEntity(entity) | Self::AttackFollowEntity(entity) => {
                let (transform, _velocity, _carried_by) = targets.get(*entity)?;
                commands.insert(Navigator {
                    target: transform.translation().xy(),
                    slow_factor: 1.0,
                });
            }
            Self::AttackEntity(entity) => {
                let (transform, velocity, carried_by) = targets.get(*entity)?;
                if carried_by.is_some() {
                    return Err(Error::Default);
                }
                let target_position = transform.translation().xy() + velocity.0;
                commands.insert((
                    Navigator {
                        target: transform.translation().xy(),
                        slow_factor: 1.0,
                    },
                    DashAttacker {
                        target: target_position,
                        ..default()
                    },
                ));
            }
        };
        Ok(())
    }

    /// When objective is unchanged, update the values in the components.
    pub fn try_update_components(
        &self,
        components: &mut ObjectivesQueryDataItem,
        targets: &Query<(&GlobalTransform, &Velocity, Option<&CarriedBy>)>,
    ) -> Result<(), Error> {
        match self {
            Self::None => {}
            Self::FollowEntity(entity) | Self::AttackFollowEntity(entity) => {
                let (transform, _velocity, _carried_by) = targets.get(*entity)?;
                if let Some(ref mut navigator) = components.navigator {
                    navigator.target = transform.translation().xy();
                }
            }
            Self::AttackEntity(entity) => {
                let (transform, velocity, carried_by) = targets.get(*entity)?;
                if carried_by.is_some() {
                    return Err(Error::Default);
                }
                let target_position = transform.translation().xy() + velocity.0;
                if let Some(ref mut navigator) = components.navigator {
                    navigator.target = target_position;
                }
                if let Some(ref mut dash_attacker) = components.dash_attacker {
                    dash_attacker.target = target_position;
                }
            }
        };
        Ok(())
    }

    /// If this objective is following an entity, return that Entity.
    pub fn get_followed_entity(&self) -> Option<Entity> {
        match self {
            Self::AttackEntity(entity)
            | Self::AttackFollowEntity(entity)
            | Self::FollowEntity(entity) => Some(*entity),
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
        targets: Query<(&GlobalTransform, &Velocity, Option<&CarriedBy>)>,
        mut commands: Commands,
    ) {
        for (mut objectives, mut components) in query.iter_mut() {
            loop {
                let result = if objectives.is_changed() {
                    objectives
                        .last()
                        .try_add_components(&mut components, &targets, &mut commands)
                } else {
                    objectives
                        .last()
                        .try_update_components(&mut components, &targets)
                };

                if result.is_ok() {
                    break;
                } else {
                    objectives.pop();
                }
            }
        }
    }

    pub fn iter(&self) -> Iter<Objective> {
        self.0.iter()
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
}
