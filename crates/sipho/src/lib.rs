pub mod objectives;
pub mod objects;
pub mod scene;
pub mod ui;

pub mod prelude {
    pub use sipho_core::prelude::*;
    pub use sipho_vfx::prelude::*;

    pub use crate::{
        objectives::{Objective, ObjectiveConfig, ObjectiveDebugger, Objectives},
        objects::{
            CarriedBy, CarryEvent, Consumer, DamageEvent, Health, InteractionConfigs, Object,
            ObjectBundle, ObjectCommands, ObjectConfig, ObjectConfigs, ObjectSpec,
        },
        ui::{Selected, Waypoint},
        SiphonowarPlugin,
    };
}

use prelude::*;

pub struct SiphonowarPlugin;
impl Plugin for SiphonowarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(window::custom_plugin()),
            CorePlugin,
            objects::ObjectsPlugin,
            objectives::ObjectivePlugin,
            scene::LoadableScenePlugin,
            ui::UiPlugin,
            sipho_vfx::VfxPlugin,
        ));
    }
}
