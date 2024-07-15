use std::fs::File;
use std::io::Write;

use bevy::{prelude::*, tasks::IoTaskPool};

use crate::prelude::*;

/// Plugin for saving and loading scenes.
pub struct LoadableScenePlugin;
impl Plugin for LoadableScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveEvent>()
            .register_type::<SaveEntity>()
            .register_type::<Name>()
            .register_type::<core::num::NonZeroU16>()
            .add_systems(PreStartup, load_system)
            .add_systems(
                FixedUpdate,
                SaveEvent::update.in_set(FixedUpdateStage::Despawn),
            );
    }
}

/// Use this to tag entities that should be saved in the scene.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct SaveEntity;

// The initial scene file will be loaded below and not change when the scene is saved
const SCENE_FILE_PATH: &str = "scenes/test.scn.ron";

pub fn load_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut load_state: ResMut<AssetLoadState>,
) {
    let scene = DynamicSceneBundle {
        // Scenes are loaded just like any other asset.
        scene: asset_server.load(SCENE_FILE_PATH),
        ..default()
    };
    load_state.track(&scene.scene);
    // "Spawning" a scene bundle creates a new entity and spawns new instances
    // of the given scene's entities as children of that entity.
    commands.spawn((Name::new("DynamicScene"), scene));
}

#[derive(Event, Debug, Clone)]
pub struct SaveEvent {
    pub path: String,
}
impl SaveEvent {
    pub fn update(
        world: &World,
        query: Query<Entity, With<SaveEntity>>,
        mut events: EventReader<SaveEvent>,
    ) {
        if let Some(event) = events.read().next() {
            let scene = DynamicSceneBuilder::from_world(world)
                .extract_entities(query.iter())
                .allow_resource::<ObjectConfigs>()
                .allow_resource::<Grid2<EntitySet>>()
                .extract_resources()
                .build();

            // Scenes can be serialized like this:
            let type_registry = world.resource::<AppTypeRegistry>();
            let serialized_scene = scene
                .serialize(&type_registry.internal.try_read().unwrap())
                .unwrap();
            // Showing the scene in the console
            info!("Saving scene: {}", serialized_scene);

            let cloned_event = event.clone();
            // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
            // as they are blocking
            // This can't work in WASM as there is no filesystem access
            #[cfg(not(target_arch = "wasm32"))]
            IoTaskPool::get()
                .spawn(async move {
                    // Write the scene RON data to file
                    File::create(format!("assets/{}", cloned_event.path))
                        .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                        .expect("Error while writing scene to file");
                })
                .detach();
        }
    }
}
