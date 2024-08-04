use crate::prelude::*;
use bevy::{ecs::query::QueryData, input::ButtonState};

use super::{elastic::SpawnElasticEvent, neighbors::NeighborsBundle};

pub struct ObjectBuilderPlugin;
impl Plugin for ObjectBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ElasticBuilder>()
            .register_type::<ObjectBuilder>()
            .add_systems(Startup, ObjectBuilder::setup)
            .add_systems(
                FixedUpdate,
                ObjectBuilder::update
                    .in_set(FixedUpdateStage::Spawn)
                    .in_set(GameStateSet::Running),
            );
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ObjectBuilder {
    pub object: Option<Object>,
}

#[derive(Bundle)]
pub struct ObjectBuilderBundle {
    name: Name,
    builder: ObjectBuilder,
    pbr: PbrBundle,
    object: Object,
    team: Team,
    position: Position,
    velocity: Velocity,
    neighbors: NeighborsBundle,
    // outline: OutlineBundle,
}
impl Default for ObjectBuilderBundle {
    fn default() -> Self {
        Self {
            name: Name::new("ObjectBuilder"),
            builder: ObjectBuilder::default(),
            pbr: PbrBundle {
                visibility: Visibility::Hidden,
                ..default()
            },
            object: Object::BuilderPreview,
            team: Team::None,
            position: Position::ZERO,
            velocity: Velocity::ZERO,
            neighbors: NeighborsBundle::default(),
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct ElasticBuilder {
    pub neighbor: Option<Entity>,
    pub tie_neighbor: Option<Entity>,
}

#[derive(Bundle)]
pub struct ElasticBuilderBundle {
    name: Name,
    builder: ElasticBuilder,
    pbr: PbrBundle,
}
impl Default for ElasticBuilderBundle {
    fn default() -> Self {
        Self {
            name: Name::new("ObjectElasticBuilder"),
            builder: ElasticBuilder::default(),
            pbr: PbrBundle {
                visibility: Visibility::Hidden,
                ..default()
            },
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ObjectBuilderQueryData {
    builder: &'static mut ObjectBuilder,
    position: &'static mut Position,
    visibility: &'static mut Visibility,
    mesh: &'static mut Handle<Mesh>,
    transform: &'static mut Transform,
    team: &'static mut Team,
    neighbors: &'static AlliedNeighbors,
}
impl ObjectBuilderQueryDataItem<'_> {
    pub fn init(&mut self, object: Object, configs: &ObjectConfigs, assets: &ObjectAssets) {
        // If not already in this object state, switch to this object.
        if self.builder.bypass_change_detection().object == Some(object) {
        } else {
            let config = configs.get(&object).unwrap();
            self.builder.object = Some(object);
            *self.visibility = Visibility::Visible;
            *self.mesh = assets.object_meshes[&object].clone();
            self.transform.scale = Vec3::splat(config.radius * 1.2);
        }
    }
    pub fn resposition(&mut self, position: Vec2) {
        self.position.0 = position;
        self.transform.translation = self.position.extend(zindex::ZOOIDS_MAX);
    }
    pub fn show(
        &mut self,
        elastic: &mut ElasticBuilderQueryDataItem,
        objects: &Query<(&Position, &PathToHead), Without<ObjectBuilder>>,
    ) {
        // Handle elastics logic.
        if let Some(neighbor) = self.neighbors.first() {
            let magnitude = neighbor.delta.length();
            if magnitude <= Elastic::MAX_LENGTH {
                let width = 6.;
                if let Ok((position, path_to_head)) = objects.get(neighbor.entity) {
                    if path_to_head.head.is_some() {
                        elastic.builder.neighbor = Some(neighbor.entity);
                        elastic.show(Elastic::get_transform(
                            self.position.0,
                            position.0,
                            neighbor.delta.length(),
                            self.transform.translation.z,
                            width,
                        ));
                        return;
                    }
                }
            }
        }

        elastic.hide();
    }

    pub fn show_tie(
        &mut self,
        elastic: &mut ElasticBuilderQueryDataItem,
        objects: &Query<(&Position, &PathToHead), Without<ObjectBuilder>>,
        position: Vec2,
    ) {
        if let Some(neighbor) = self.neighbors.first() {
            if let Some(tie_neighbor) = elastic.builder.tie_neighbor {
                if (neighbor.entity) != tie_neighbor {
                    elastic.builder.neighbor = Some(neighbor.entity);
                }
            } else {
                elastic.builder.tie_neighbor = Some(neighbor.entity);
            }

            if let Some(tie_neighbor) = elastic.builder.tie_neighbor {
                if let Some(neighbor) = elastic.builder.neighbor {
                    if let (Ok((&position1, _)), Ok((&position2, _))) =
                        (objects.get(tie_neighbor), objects.get(neighbor))
                    {
                        let width = 6.;
                        let magnitude = position1.distance(*position2);

                        // if magnitude <= Elastic::MAX_LENGTH {
                        elastic.show(Elastic::get_transform(
                            *position1,
                            *position2,
                            magnitude,
                            self.transform.translation.z,
                            width,
                        ));
                        // }
                    }
                } else if let Ok((&tie_position, _)) = objects.get(tie_neighbor) {
                    let width = 6.;
                    let magnitude = tie_position.distance(position);

                    // if magnitude <= Elastic::MAX_LENGTH {
                    elastic.show(Elastic::get_transform(
                        *tie_position,
                        position,
                        magnitude,
                        self.transform.translation.z,
                        width,
                    ));
                    // }
                }
            }
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ElasticBuilderQueryData {
    builder: &'static mut ElasticBuilder,
    visibility: &'static mut Visibility,
    transform: &'static mut Transform,
}
impl ElasticBuilderQueryDataItem<'_> {
    pub fn show(&mut self, transform: Transform) {
        *self.visibility = Visibility::Visible;
        *self.transform = transform;
    }
    pub fn hide(&mut self) {
        *self.visibility = Visibility::Hidden;
        self.builder.neighbor = None;
        self.builder.tie_neighbor = None;
    }
}

impl ObjectBuilder {
    pub fn setup(mut commands: Commands, assets: Res<ObjectAssets>) {
        commands
            .spawn(ObjectBuilderBundle::default())
            .insert(assets.builder_material.clone());

        commands.spawn(ElasticBuilderBundle::default()).insert((
            assets.builder_material.clone(),
            assets.connector_mesh.clone(),
        ));
    }

    pub fn get_buildable_object(action: ControlAction) -> Option<Object> {
        Some(match action {
            ControlAction::Worker => Object::Worker,
            ControlAction::Shocker => Object::Shocker,
            ControlAction::Armor => Object::Armor,
            _ => return None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut builder: Query<ObjectBuilderQueryData, Without<ElasticBuilder>>,
        objects: Query<(&Position, &PathToHead), Without<ObjectBuilder>>,
        mut elastic_builder: Query<ElasticBuilderQueryData, Without<ObjectBuilder>>,
        mut events: EventReader<ControlEvent>,
        assets: Res<ObjectAssets>,
        mut commands: ObjectCommands,
        team_config: Res<TeamConfig>,
        object_configs: Res<ObjectConfigs>,
        mut elastic_events: EventWriter<SpawnElasticEvent>,
        mut audio: EventWriter<AudioEvent>,
        mut frame_count: Local<usize>,
    ) {
        let mut builder = builder.single_mut();
        let mut elastic_builder = elastic_builder.single_mut();
        if team_config.is_changed() {
            *builder.team = team_config.player_team;
        }
        for event in events.read() {
            if let Some(object) = Self::get_buildable_object(event.action) {
                match event.state {
                    ButtonState::Pressed => {
                        if *frame_count == 0 {
                            builder.init(object, &object_configs, &assets);
                            builder.resposition(event.position);
                        }
                        if *frame_count > 1 {
                            builder.resposition(event.position);
                            builder.show(&mut elastic_builder, &objects);
                        }
                        *frame_count += 1;
                    }
                    ButtonState::Released => {
                        *frame_count = 0;
                        if let Some(neighbor) = elastic_builder.builder.neighbor {
                            if let Ok((_position, path_to_head)) = objects.get(neighbor) {
                                if let Some(head) = path_to_head.head {
                                    if commands.try_consume(head, 1).is_ok() {
                                        if let Some(entity_commands) = commands.spawn(ObjectSpec {
                                            object,
                                            position: Position(event.position),
                                            team: team_config.player_team,
                                            ..default()
                                        }) {
                                            elastic_events.send(SpawnElasticEvent {
                                                elastic: Elastic((neighbor, entity_commands.id())),
                                                team: team_config.player_team,
                                            });
                                            audio.send(AudioEvent {
                                                sample: AudioSample::RandomBubble,
                                                position: Some(event.position),
                                                ..default()
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        builder.builder.object = None;
                        *builder.visibility = Visibility::Hidden;
                        elastic_builder.hide();
                    }
                }
            } else if event.action == ControlAction::Tie {
                match event.state {
                    ButtonState::Pressed => {
                        builder.resposition(event.position);
                        if *frame_count > 1 {
                            builder.show_tie(&mut elastic_builder, &objects, event.position);
                        }
                        *frame_count += 1;
                    }
                    ButtonState::Released => {
                        *frame_count = 0;
                        if let (Some(neighbor1), Some(neighbor2)) = (
                            elastic_builder.builder.neighbor,
                            elastic_builder.builder.tie_neighbor,
                        ) {
                            elastic_events.send(SpawnElasticEvent {
                                elastic: Elastic((neighbor1, neighbor2)),
                                team: team_config.player_team,
                            });
                            audio.send(AudioEvent {
                                sample: AudioSample::RandomBubble,
                                position: Some(event.position),
                                ..default()
                            });
                        }

                        builder.builder.object = None;
                        *builder.visibility = Visibility::Hidden;
                        elastic_builder.hide();
                    }
                }
            }
        }
    }
    // Have fake objects that query the grid for nearest neighbor.
}
