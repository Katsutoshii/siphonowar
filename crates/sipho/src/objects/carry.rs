use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

use super::zooid_head::NearestZooidHead;

/// Plugin for picking up items and carrying them.
/// 1) When one entity begins carrying the other, their velocity is zeroed out.
/// 2) At each step, we guarantee that all carrying entities have the same acceleration and velocity.
pub struct CarryPlugin;
impl Plugin for CarryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CarryEvent>().add_systems(
            FixedUpdate,
            (CarryEvent::update, CarriedBy::update)
                .chain()
                .in_set(SystemStage::PostCompute),
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
    pub carried_by: Option<&'static mut CarriedBy>,
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
                let carried = query.get_mut(event.carried).unwrap();

                if let Some(mut carried_by) = carried.carried_by {
                    carried_by.push(event.carrier);
                } else {
                    commands
                        .entity(event.carried)
                        .insert(CarriedBy::new(event.carrier));
                }

                commands
                    .entity(event.carrier)
                    .set_parent_in_place(event.carried);
            };
            let mut carrier = query.get_mut(event.carrier).unwrap();
            carrier.objectives.clear();

            if let Some(NearestZooidHead {
                entity: Some(entity),
            }) = carrier.nearest_head
            {
                carrier.objectives.push(Objective::FollowEntity(*entity));
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
    pub fn update(
        mut carried: Query<(Entity, &mut CarriedBy)>,
        mut carriers_query: Query<Entity, Without<CarriedBy>>,
        mut commands: Commands,
    ) {
        for (entity, mut carried_by) in &mut carried {
            let mut valid_carriers = Vec::default();

            for &carrier in carried_by.iter() {
                if let Ok(carrier) = carriers_query.get_mut(carrier) {
                    valid_carriers.push(carrier);
                    break;
                } else {
                    warn!("No carriers!");
                }
            }

            carried_by.0 = valid_carriers;
            if carried_by.is_empty() {
                info!("Delete CarriedBy");
                commands.entity(entity).remove::<Self>();
            }
        }
    }
}
