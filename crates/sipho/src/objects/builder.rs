use crate::prelude::*;
use bevy::input::ButtonState;

use super::{elastic::SpawnElasticEvent, neighbors::NeighborsBundle};

pub struct ObjectBuilderPlugin;
impl Plugin for ObjectBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ObjectBuilder::setup).add_systems(
            FixedUpdate,
            ObjectBuilder::update
                .in_set(FixedUpdateStage::AI)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Component, Default)]
pub struct ObjectBuilder {
    pub object: Option<Object>,
}
#[derive(Component, Default)]
pub struct ObjectElasticBuilder {
    pub neighbor: Option<Entity>,
}

impl ObjectBuilder {
    pub fn setup(mut commands: Commands, assets: Res<ObjectAssets>) {
        commands.spawn((
            Name::new("ObjectBuilder"),
            ObjectBuilder::default(),
            PbrBundle {
                visibility: Visibility::Hidden,
                material: assets.builder_material.clone(),
                ..default()
            },
            Object::BuilderPreview,
            Team::None,
            Position::ZERO,
            Velocity::ZERO,
            NeighborsBundle::default(),
        ));
        commands.spawn((
            ObjectElasticBuilder::default(),
            Name::new("ObjectElasticBuilder"),
            PbrBundle {
                visibility: Visibility::Hidden,
                material: assets.builder_material.clone(),
                mesh: assets.connector_mesh.clone(),
                ..default()
            },
        ));
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut builder: Query<
            (
                &mut ObjectBuilder,
                &mut Position,
                &mut Visibility,
                &mut Handle<Mesh>,
                &mut Transform,
                &mut Team,
                &AlliedNeighbors,
            ),
            Without<ObjectElasticBuilder>,
        >,
        positions: Query<&Position, Without<ObjectBuilder>>,
        mut elastic_builder: Query<
            (&mut ObjectElasticBuilder, &mut Visibility, &mut Transform),
            Without<ObjectBuilder>,
        >,
        mut events: EventReader<ControlEvent>,
        assets: Res<ObjectAssets>,
        mut commands: ObjectCommands,
        team_config: Res<TeamConfig>,
        object_configs: Res<ObjectConfigs>,
        mut elastic_events: EventWriter<SpawnElasticEvent>,
    ) {
        let (
            mut builder,
            mut position,
            mut visibility,
            mut mesh,
            mut transform,
            mut team,
            neighbors,
        ) = builder.single_mut();
        let (mut elastic_builder, mut elastic_visibility, mut elastic_transform) =
            elastic_builder.single_mut();
        if team_config.is_changed() {
            *team = team_config.player_team;
        }
        for event in events.read() {
            match event {
                ControlEvent {
                    action: ControlAction::BuildWorker,
                    state: ButtonState::Pressed,
                    ..
                } => {
                    // If not already in worker state, switch to worker.
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

                    position.0 = event.position;
                    transform.translation = event.position.extend(zindex::ZOOIDS_MAX);

                    // Handle elastics logic.
                    if let Some(neighbor) = neighbors.first() {
                        let magnitude = neighbor.delta.length();
                        if magnitude <= Elastic::MAX_LENGTH {
                            *elastic_visibility = Visibility::Visible;
                            *elastic_transform = Elastic::get_transform(
                                position.0,
                                positions.get(neighbor.entity).unwrap().0,
                                neighbor.delta.length(),
                                transform.translation.z,
                                4.,
                            );
                            elastic_builder.neighbor = Some(neighbor.entity);
                        } else {
                            *elastic_visibility = Visibility::Hidden;
                            elastic_builder.neighbor = None;
                        }
                    } else {
                        *elastic_visibility = Visibility::Hidden;
                        elastic_builder.neighbor = None;
                    }
                }
                // On release, spawn.
                ControlEvent {
                    action: ControlAction::BuildWorker,
                    state: ButtonState::Released,
                    ..
                } => {
                    if let Some(entity_commands) = commands.spawn(ObjectSpec {
                        object: Object::Worker,
                        position: event.position,
                        team: team_config.player_team,
                        ..default()
                    }) {
                        if let Some(neighbor) = elastic_builder.neighbor {
                            elastic_events.send(SpawnElasticEvent {
                                elastic: Elastic((neighbor, entity_commands.id())),
                                team: team_config.player_team,
                            });
                        }
                    }
                    builder.object = None;
                    *visibility = Visibility::Hidden;
                    *elastic_visibility = Visibility::Hidden;
                }
                _ => {}
            }
        }
    }
    // Have fake objects that query the grid for nearest neighbor.
}
