pub mod ambience;
pub mod camera;
pub mod creatures;
pub mod objectives;
pub mod objects;
pub mod scene;
pub mod terrain;
pub mod ui;

pub mod prelude {
    pub use crate::{
        objectives::{
            Cooldown, Navigator, Objective, ObjectiveConfig, ObjectiveDebugger, Objectives, Stunned,
        },
        objects::*,
        ui::{Selectable, Selected, Waypoint},
        SiphonowarPlugin,
    };
    pub use bevy_newtonian2d::*;
    pub use sipho_core::prelude::*;
    pub use sipho_sfx::prelude::*;
    pub use sipho_vfx::prelude::*;
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
                .set(window::custom_plugin())
                .set(ImagePlugin::default_linear()),
            // OutlinePlugin,
            CorePlugin,
            camera::CameraPlugin,
            objects::ObjectsPlugin,
            creatures::CreaturePlugin,
            objectives::ObjectivePlugin,
            scene::LoadableScenePlugin,
            ui::UiPlugin,
            sipho_vfx::VfxPlugin,
            sipho_sfx::SiphoSfxPlugin,
            terrain::TerrainPlugin,
            ambience::AmbiencePlugin,
        ));
    }
}
