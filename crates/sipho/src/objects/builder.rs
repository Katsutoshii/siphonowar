use crate::prelude::*;
use bevy::input::ButtonState;

pub struct ObjectBuilderPlugin;
impl Plugin for ObjectBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ObjectBuilder::setup).add_systems(
            FixedUpdate,
            ObjectBuilder::update.in_set(FixedUpdateStage::Spawn),
        );
    }
}

#[derive(Component, Default)]
pub struct ObjectBuilder {
    pub object: Option<Object>,
}
impl ObjectBuilder {
    pub fn setup(mut commands: Commands, assets: Res<ObjectAssets>) {
        commands.spawn((
            ObjectBuilder::default(),
            PbrBundle {
                visibility: Visibility::Hidden,
                material: assets.builder_material.clone(),
                ..default()
            },
        ));
    }
    pub fn update(
        mut builder: Query<(
            &mut ObjectBuilder,
            &mut Visibility,
            &mut Handle<Mesh>,
            &mut Transform,
        )>,
        mut events: EventReader<ControlEvent>,
        assets: Res<ObjectAssets>,
        mut commands: ObjectCommands,
        team_config: Res<TeamConfig>,
        object_configs: Res<ObjectConfigs>,
    ) {
        let (mut builder, mut visibility, mut mesh, mut transform) = builder.single_mut();
        for event in events.read() {
            match event {
                ControlEvent {
                    action: ControlAction::BuildWorker,
                    state: ButtonState::Pressed,
                    ..
                } => {
                    if !matches!(
                        builder.bypass_change_detection().object,
                        Some(Object::Worker)
                    ) {
                        let config = object_configs.get(&Object::Worker).unwrap();
                        builder.object = Some(Object::Worker);
                        *visibility = Visibility::Visible;
                        *mesh = assets.worker_mesh.clone();
                        transform.scale = Vec3::splat(config.radius);
                    }

                    transform.translation = event.position.extend(zindex::ZOOIDS_MAX);
                }
                ControlEvent {
                    action: ControlAction::BuildWorker,
                    state: ButtonState::Released,
                    ..
                } => {
                    commands.spawn(ObjectSpec {
                        object: Object::Worker,
                        position: event.position,
                        team: team_config.player_team,
                        ..default()
                    });
                    builder.object = None;
                    *visibility = Visibility::Hidden;
                }
                _ => {}
            }
        }
    }
    // Have fake objects that query the grid for nearest neighbor.
}
