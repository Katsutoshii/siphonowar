use crate::prelude::*;

mod assets;
mod carry;
mod commands;
mod config;
mod consumer;
mod damage;
mod elastic;
mod neighbors;
mod object;
mod plankton;
mod zooid_head;
mod zooid_worker;

pub use {
    assets::ObjectAssets,
    carry::{CarriedBy, CarryEvent},
    commands::{ObjectBundle, ObjectCommands, ObjectSpec},
    config::{InteractionConfig, InteractionConfigs, ObjectConfig, ObjectConfigs},
    consumer::Consumer,
    damage::{DamageEvent, Health},
    elastic::{AttachedTo, Elastic, ElasticPlugin},
    neighbors::{AlliedNeighbors, CollidingNeighbors, EnemyNeighbors},
    object::Object,
};

/// Plugin for running zooids simulation.
pub struct ObjectsPlugin;
impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            config::ObjectConfigPlugin,
            consumer::ConsumerPlugin,
            carry::CarryPlugin,
            neighbors::NeighborsPlugin,
            zooid_head::ZooidHeadPlugin,
            zooid_worker::ZooidWorkerPlugin,
            elastic::ElasticPlugin,
            plankton::PlanktonPlugin,
            object::ObjectPlugin,
            damage::DamagePlugin,
        ))
        .init_resource::<ObjectAssets>();
    }
}
