use crate::prelude::*;
use bevy::color::palettes::css::{PURPLE, SEA_GREEN};
use bevy::utils::HashMap;

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct ObjectAssets {
    pub object_meshes: HashMap<Object, Handle<Mesh>>,
    pub connector_mesh: Handle<Mesh>,
    pub team_materials: Vec<TeamMaterials>,
    pub builder_material: Handle<StandardMaterial>,
    pub food_material: Handle<StandardMaterial>,
    pub crystal_material: Handle<StandardMaterial>,
}
impl ObjectAssets {
    pub fn get_team_material(&self, team: Team) -> TeamMaterials {
        self.team_materials.get(team as usize).unwrap().clone()
    }
}
impl FromWorld for ObjectAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            object_meshes: {
                let mut meshes = HashMap::new();
                meshes.insert(
                    Object::Worker,
                    world.load_asset("models/zooids/worker/worker.glb#Mesh0/Primitive0"),
                );
                meshes.insert(Object::Head, meshes[&Object::Worker].clone());
                meshes.insert(Object::Plankton, meshes[&Object::Worker].clone());
                meshes.insert(Object::Food, meshes[&Object::Worker].clone());
                meshes.insert(
                    Object::Gem,
                    world.load_asset("models/minerals/Gem.glb#Mesh0/Primitive0"),
                );
                meshes.insert(
                    Object::Shocker,
                    world.load_asset("models/zooids/shocker/shocker.glb#Mesh0/Primitive0"),
                );
                meshes.insert(
                    Object::Armor,
                    world.load_asset("models/zooids/armor/armor.glb#Mesh0/Primitive0"),
                );
                meshes
            },

            connector_mesh: world.load_asset("models/connector/connector.gltf#Mesh0/Primitive0"),
            team_materials: {
                let mut materials = world.assets::<StandardMaterial>();
                Team::COLORS
                    .iter()
                    .map(|color| TeamMaterials::new(*color, &mut materials))
                    .collect()
            },
            builder_material: world.append_asset(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 0.8, 0.35),
                emissive: Color::srgba(1.0, 1.0, 0.8, 0.35).into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            food_material: world.append_asset(StandardMaterial {
                base_color: SEA_GREEN.into(),
                emissive: SEA_GREEN.into(),
                perceptual_roughness: 1.0,
                alpha_mode: AlphaMode::Opaque,
                ..default()
            }),
            crystal_material: world.append_asset(StandardMaterial {
                base_color: PURPLE.into(),
                emissive: PURPLE.into(),
                perceptual_roughness: 1.0,
                alpha_mode: AlphaMode::Opaque,
                ..default()
            }),
        }
    }
}
