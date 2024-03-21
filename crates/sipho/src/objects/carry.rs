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
            (CarryEvent::update, Carrier::update, CarriedBy::update)
                .in_set(SystemStage::PostCompute)
                .after(Objectives::update)
                .chain(),
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
            let carried_velocity = {
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
                    .insert(Carrier::new(event.carried));
                *carried.velocity
            };

            let mut carrier = query.get_mut(event.carrier).unwrap();
            *carrier.velocity = carried_velocity;
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

#[derive(Component, Debug, Copy, Clone)]
pub struct Carrier {
    pub entity: Entity,
}
impl Carrier {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
    /// Cleanup invalid carriers.
    pub fn update(
        mut carriers: Query<(Entity, &Carrier)>,
        carried: Query<&Velocity, With<CarriedBy>>,
        mut commands: Commands,
    ) {
        for (entity, carrier) in &mut carriers {
            if carried.get(carrier.entity).is_err() {
                info!("Carrier removed.");
                commands.entity(entity).remove::<Self>();
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
    /// Accululate acceleration from all carriers.
    pub fn update(
        mut carried: Query<(Entity, &mut Self, &Velocity, &mut Acceleration), Without<Carrier>>,
        mut carriers_query: Query<(&mut Velocity, &mut Acceleration), With<Carrier>>,
        mut commands: Commands,
    ) {
        for (entity, mut carriers, velocity, mut acceleration) in &mut carried {
            let mut valid_carriers = Vec::default();
            // *acceleration = Acceleration::ZERO;

            // Sum all accelerations.
            for &carrier in carriers.iter() {
                if let Ok((_carrier_velocity, carrier_acceleration)) = carriers_query.get(carrier) {
                    valid_carriers.push(carrier);
                    *acceleration += *carrier_acceleration;
                    break;
                } else {
                    warn!("No carriers!");
                }
            }

            // Set all carrier accelerations to the parent's acceleration.
            for &carrier in &valid_carriers {
                if let Ok((mut carrier_velocity, mut carrier_acceleration)) =
                    carriers_query.get_mut(carrier)
                {
                    *carrier_velocity = *velocity;
                    *carrier_acceleration = *acceleration;
                    // *carrier_acceleration = Acceleration::ZERO;
                }
            }

            carriers.0 = valid_carriers;
            if carriers.is_empty() {
                info!("Delete CarriedBy");
                commands.entity(entity).remove::<Self>();
            }
        }
    }
}
