pub mod objectives;
pub mod objects;
pub mod scene;
pub mod ui;

pub mod prelude {
    pub use sipho_core::prelude::*;
    pub use sipho_vfx::prelude::*;

    pub use crate::{
        objectives::{Objective, ObjectiveConfig, ObjectiveDebugger, Objectives},
        objects::*,
        ui::{Selected, Waypoint},
        SiphonowarPlugin,
    };
}

use prelude::*;
use std::f32::consts::PI;

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
            CorePlugin,
            objects::ObjectsPlugin,
            objectives::ObjectivePlugin,
            scene::LoadableScenePlugin,
            ui::UiPlugin,
            sipho_vfx::VfxPlugin,
        ))
        .add_systems(Startup, spawn_gltf);
    }
}

fn spawn_gltf(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawn gltf");
    commands.spawn((
        Name::new("Rocks"),
        SceneBundle {
            scene: asset_server.load("models/rocks/RocksLowPoly.gltf#Scene0"),
            transform: Transform {
                scale: Vec3 {
                    x: 8.,
                    y: 1.5,
                    z: 8.,
                },
                translation: Vec3 {
                    x: -150.,
                    y: 490.,
                    z: -10.,
                },
                rotation: Quat::from_axis_angle(Vec3::X, PI / 2.),
            },
            ..default()
        },
    ));
}
