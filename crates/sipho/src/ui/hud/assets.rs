use crate::prelude::*;

use super::minimap::MinimapUiMaterial;

#[derive(Resource)]
pub struct HudAssets {
    pub minimap_material: Handle<MinimapUiMaterial>,
}

impl FromWorld for HudAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            minimap_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<MinimapUiMaterial>>()
                    .unwrap();
                materials.add(MinimapUiMaterial::default())
            },
        }
    }
}
