use crate::prelude::*;

pub mod minimap;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((minimap::MinimapPlugin,));
    }
}
