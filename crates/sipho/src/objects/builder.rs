use crate::prelude::*;

pub struct ObjectBuilderPlugin;
impl Plugin for ObjectBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update);
    }
}

pub fn update() {}
// Have fake objects that query the grid for nearest neighbor.
