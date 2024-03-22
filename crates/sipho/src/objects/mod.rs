use crate::prelude::*;
use bevy::prelude::*;

use self::{
    carry::CarryPlugin, damage::DamagePlugin, neighbors::NeighborsPlugin, object::ObjectPlugin,
    objective::ObjectivePlugin, plankton::PlanktonPlugin, zooid_head::ZooidHeadPlugin,
    zooid_worker::ZooidWorkerPlugin,
};
pub use self::{
    carry::{CarriedBy, CarryEvent},
    commands::{ObjectCommands, ObjectSpec},
    config::{
        InteractionConfig, InteractionConfigs, ObjectConfig, ObjectConfigs, TestInteractionConfigs,
    },
    damage::{DamageEvent, Health},
    object::Object,
    objective::{Objective, ObjectiveConfig, ObjectiveDebugger, Objectives},
};

/// Plugin for running zooids simulation.
pub struct ObjectsPlugin;
impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CarryPlugin,
            NeighborsPlugin,
            ObjectivePlugin,
            ZooidHeadPlugin,
            ZooidWorkerPlugin,
            PlanktonPlugin,
            ObjectPlugin,
            DamagePlugin,
        ))
        .init_resource::<ObjectAssets>()
        .configure_sets(FixedUpdate, SystemStage::get_config());
    }
}

mod carry;
mod commands;
mod config;
mod damage;
mod neighbors;
mod object;
mod objective;
mod plankton;
mod zooid_head;
mod zooid_worker;

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
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(Circle::default()))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            team_materials: Team::COLORS
                .iter()
                .map(|color| TeamMaterials::new(*color, &mut materials))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_update() {}
}
