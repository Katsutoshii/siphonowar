use crate::prelude::*;

use super::minimap::MinimapUiMaterial;

#[derive(Resource)]
pub struct HudAssets {
    pub minimap_material: Handle<MinimapUiMaterial>,
}

impl FromWorld for HudAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            minimap_material: { world.add_asset(MinimapUiMaterial::default()) },
        }
    }
}
