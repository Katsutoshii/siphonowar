use crate::prelude::*;

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct ObjectAssets {
    pub worker_mesh: Handle<Mesh>,
    pub connector_mesh: Handle<Mesh>,
    team_materials: Vec<TeamMaterials>,
}
impl ObjectAssets {
    pub fn get_team_material(&self, team: Team) -> TeamMaterials {
        self.team_materials.get(team as usize).unwrap().clone()
    }
}
impl FromWorld for ObjectAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            worker_mesh: {
                let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                asset_server.load("models/zooids/worker/worker.glb#Mesh0/Primitive0")
            },
            connector_mesh: {
                let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                asset_server.load("models/connector/connector.gltf#Mesh0/Primitive0")
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
