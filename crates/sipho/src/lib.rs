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

fn spawn_gltf(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // info!("Spawn gltf");
    // // note that we have to include the `Scene0` label
    // // let mesh = asset_server.load("models/triangle/triangle.gltf#Mesh0/Primitive0");
    // let mesh = meshes.add(Mesh::from(RegularPolygon {
    //     circumcircle: Circle {
    //         radius: 2f32.sqrt() / 2.,
    //     },
    //     sides: 3,
    // }));
    // let material = materials.add(StandardMaterial::from(Color::TURQUOISE));
    // // to position our 3d model, simply use the Transform
    // // in the SceneBundle
    // commands.spawn((
    //     Name::new("test"),
    //     PbrBundle {
    //         mesh,
    //         transform: Transform::default()
    //             .with_scale(Vec2::splat(10.5).extend(1.))
    //             .with_translation(Vec3 {
    //                 x: 0.0,
    //                 y: 0.0,
    //                 z: 2000.,
    //             }),
    //         material,
    //         ..default()
    //     },
    // ));
}
