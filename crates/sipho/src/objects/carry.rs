use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

use super::zooid_head::NearestZooidHead;

/// Plugin for picking up items and carrying them.
/// 1) When one entity begins carrying the other, their velocity is zeroed out.
/// 2) At each step, we guarantee that all carrying entities have the same force and velocity.
pub struct CarryPlugin;
impl Plugin for CarryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CarryEvent>().add_systems(
            FixedUpdate,
            (CarryEvent::update, CarriedBy::update)
                .chain()
                .in_set(FixedUpdateStage::AccumulateForces)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Event, Debug)]
pub struct CarryEvent {
    pub carrier: Entity,
    pub carried: Entity,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CarriedByQueryData {
    pub entity: Entity,
    pub carried_by: &'static mut CarriedBy,
    pub objectives: &'static mut Objectives,
    pub nearest_head: Option<&'static NearestZooidHead>,
    pub velocity: &'static mut Velocity,
}

impl CarryEvent {
    /// Set up the carry.
    pub fn update(
        mut events: EventReader<Self>,
        mut query: Query<CarriedByQueryData>,
        mut commands: Commands,
    ) {
        for event in events.read() {
            {
                let mut carried = query.get_mut(event.carried).unwrap();
                carried.carried_by.push(event.carrier);
                commands
                    .entity(event.carrier)
                    .set_parent_in_place(event.carried);
            }
            {
                let mut carrier = query.get_mut(event.carrier).unwrap();

                if let Some(NearestZooidHead {
                    entity: Some(entity),
                }) = carrier.nearest_head
                {
                    carrier.objectives.clear();
                    carrier.objectives.push(Objective::FollowEntity(*entity));
                }
            }
        }
    }
}

#[derive(Component, Deref, DerefMut, Default, Clone, Debug)]
pub struct CarriedBy(pub Vec<Entity>);
impl CarriedBy {
    pub fn new(entity: Entity) -> Self {
        Self(vec![entity])
    }
    pub fn update(mut carried: Query<&mut CarriedBy>, carriers: Query<Entity>) {
        for mut carried_by in &mut carried {
            carried_by.retain(|&carrier| carriers.get(carrier).is_ok());
        }
    }
}
