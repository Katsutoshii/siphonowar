use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::objects::{InteractionConfig, ObjectConfig, TestInteractionConfigs};
use crate::prelude::*;

pub struct ConfigPlugin;
impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Vec2>()
            .register_type::<Team>()
            .register_type::<Object>()
            .register_type::<ObjectiveConfig>()
            .register_type::<PhysicsMaterialType>()
            .register_type::<InteractionConfig>()
            .register_type::<TestInteractionConfigs>()
            .register_type::<HashMap<PhysicsMaterialType, InteractionConfig>>()
            .register_type::<HashMap<Object, ObjectConfig>>()
            .register_type::<HashMap<Object, InteractionConfig>>()
            .register_type::<ObjectConfig>()
            .register_type::<ObjectConfigs>()
            .register_type::<InteractionConfigs>()
            .register_type::<Configs>();
    }
}

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Configs {
    // Specify which team the player controls.
    pub player_team: Team,
    pub visibility_radius: u16,
    pub fog_radius: u16,
    pub window_size: Vec2,
    pub cursor_sensitivity: f32,

    // Configs per object type.
    pub objects: ObjectConfigs,
}
