use crate::prelude::*;

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
    carry::{CarriedBy, CarryEvent},
    commands::{ObjectBundle, ObjectCommands, ObjectSpec},
    config::{InteractionConfig, InteractionConfigs, ObjectConfig, ObjectConfigs},
    consumer::Consumer,
    damage::{DamageEvent, Health},
    elastic::ElasticPlugin,
    neighbors::CollidingNeighbors,
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

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct ObjectAssets {
    pub mesh: Handle<Mesh>,
    team_materials: Vec<TeamMaterials>,
}
impl ObjectAssets {
    fn get_team_material(&self, team: Team) -> TeamMaterials {
        self.team_materials.get(team as usize).unwrap().clone()
    }
}
impl FromWorld for ObjectAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: {
                let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                meshes.add(Mesh::from(Sphere { radius: 0.5 }))
            },
            team_materials: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                Team::COLORS
                    .iter()
                    .map(|color| TeamMaterials::new(*color, &mut materials))
                    .collect()
            },
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_update() {}
}
