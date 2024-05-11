use crate::prelude::*;

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct ObjectAssets {
    pub worker_mesh: Handle<Mesh>,
    pub shocker_mesh: Handle<Mesh>,
    pub armor_mesh: Handle<Mesh>,
    pub connector_mesh: Handle<Mesh>,
    pub team_materials: Vec<TeamMaterials>,
    pub white_material: Handle<StandardMaterial>,
    pub builder_material: Handle<StandardMaterial>,
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
            shocker_mesh: {
                let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                asset_server.load("models/zooids/shocker/shocker.glb#Mesh0/Primitive0")
            },
            armor_mesh: {
                let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                asset_server.load("models/zooids/armor/armor.glb#Mesh0/Primitive0")
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
            white_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial::from(Color::WHITE.with_a(0.25)))
            },
            builder_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial {
                    base_color: Color::YELLOW.with_a(0.25),
                    emissive: Color::YELLOW,
                    ..default()
                })
            },
        }
    }
}
