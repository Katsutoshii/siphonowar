use crate::prelude::*;

pub struct NavigatorPlugin;
impl Plugin for NavigatorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Navigator>();
    }
}

#[derive(Reflect)]
pub enum NavTarget {
    Entity(Entity),
    Position(Vec2),
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Navigator {
    target: NavTarget,
}
