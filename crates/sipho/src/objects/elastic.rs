use crate::prelude::*;
use bevy::ecs::system::{EntityCommands, QueryLens, SystemParam};
use bevy::utils::smallvec::SmallVec;
use bevy::utils::FloatOrd;
use sipho_core::grid::fog::FogConfig;

use super::ObjectAssets;

pub struct ElasticPlugin;
impl Plugin for ElasticPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnElasticEvent>()
            .register_type::<Elastic>()
            .add_systems(
                FixedUpdate,
                (
                    (
                        Elastic::tie_cursor,
                        Elastic::tie_selection,
                        SpawnElasticEvent::update,
                    )
                        .chain(),
                    (Elastic::update).in_set(FixedUpdateStage::PostSpawn),
                )
                    .in_set(GameStateSet::Running),
            );
    }
}

#[derive(Event)]
pub struct SpawnElasticEvent {
    pub elastic: Elastic,
    pub team: Team,
}
impl SpawnElasticEvent {
    pub fn update(mut events: EventReader<SpawnElasticEvent>, mut commands: ElasticCommands) {
        for event in events.read() {
            let Elastic((entity1, entity2)) = event.elastic;
            commands.tie(entity1, entity2, event.team);
        }
    }
}

#[derive(Component, Debug, Default, DerefMut, Deref)]
pub struct AttachedTo(pub SmallVec<[Entity; 8]>);

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct Elastic(pub (Entity, Entity));
impl Default for Elastic {
    fn default() -> Self {
        Self((Entity::PLACEHOLDER, Entity::PLACEHOLDER))
    }
}
impl Elastic {
    pub fn first(&self) -> Entity {
        self.0 .0
    }
    pub fn second(&self) -> Entity {
        self.0 .1
    }
}

#[derive(SystemParam)]
pub struct ElasticCommands<'w, 's> {
    commands: Commands<'w, 's>,
    attachments: Query<'w, 's, &'static mut AttachedTo>,
    positions: Query<'w, 's, &'static Position>,
    assets: Res<'w, ObjectAssets>,
}
impl ElasticCommands<'_, '_> {
    pub fn attachments(&mut self) -> QueryLens<&AttachedTo> {
        self.attachments.transmute_lens()
    }
    pub fn tie(&mut self, entity1: Entity, entity2: Entity, team: Team) -> Option<EntityCommands> {
        for pair in [(entity1, entity2), (entity2, entity1)] {
            let (entity1, entity2) = pair;
            if let Ok(ref mut attached_to) = self.attachments.get_mut(entity1) {
                if attached_to.contains(&entity2) {
                    info!("Already attached");
                    return None;
                }
                attached_to.push(entity2);
            }
        }

        let position1 = self.positions.get(entity1).unwrap();
        let position2 = self.positions.get(entity2).unwrap();
        let magnitude = position1.distance(position2.0);

        let commands = self.commands.spawn(ElasticBundle {
            elastic: Elastic((entity1, entity2)),
            pbr: PbrBundle {
                mesh: self.assets.connector_mesh.clone(),
                material: self.assets.get_team_material(team).background,
                transform: Elastic::get_transform(
                    position1.0,
                    position2.0,
                    magnitude,
                    zindex::ZOOIDS_MIN,
                ),
                ..default()
            },
            ..default()
        });
        Some(commands)
    }
}

#[derive(Bundle)]
pub struct ElasticBundle {
    pub elastic: Elastic,
    pub pbr: PbrBundle,
    pub name: Name,
}
impl Default for ElasticBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Elastic"),
            pbr: PbrBundle::default(),
            elastic: Elastic::default(),
        }
    }
}
impl Elastic {
    pub fn get_transform(
        position1: Vec2,
        position2: Vec2,
        magnitude: f32,
        depth: f32,
    ) -> Transform {
        let width = 4.;
        let delta = position2 - position1;
        Transform {
            translation: ((position1 + position2) / 2.).extend(depth),
            scale: Vec3::new(magnitude / 2., width, width),
            rotation: Quat::from_axis_angle(Vec3::Z, delta.to_angle()),
        }
    }
    pub fn tie_cursor(
        mut control_events: EventReader<ControlEvent>,
        positions: Query<&Position>,
        config: Res<FogConfig>,
        grid: Res<Grid2<TeamEntitySets>>,
        mut last_entity: Local<Option<Entity>>,
        mut events: EventWriter<SpawnElasticEvent>,
    ) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::TieCursor) {
                let entities = grid.get_entities_in_radius(
                    control_event.position,
                    32.0,
                    &[config.player_team],
                );
                let mut dudes: Vec<Entity> = entities.iter().copied().collect();

                dudes.sort_by_key(|&entity| {
                    let position = positions.get(entity).unwrap();
                    FloatOrd(Vec2::distance_squared(control_event.position, position.0))
                });
                if let Some(&dude) = dudes.first() {
                    if let Some(last_entity) = *last_entity {
                        if last_entity != dude {
                            events.send(SpawnElasticEvent {
                                elastic: Elastic((last_entity, dude)),
                                team: config.player_team,
                            });
                        }
                    }
                    *last_entity = Some(dude);
                }
            }
            if control_event.is_released(ControlAction::TieCursor) {
                *last_entity = None;
            }
        }
    }
    pub fn tie_selection(
        mut control_events: EventReader<ControlEvent>,
        mut query: Query<(Entity, &Selected)>,
        config: Res<FogConfig>,
        mut events: EventWriter<SpawnElasticEvent>,
    ) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::TieSelection) {
                // Collect entities to tie together.
                let mut entities = vec![];
                for (entity, selected) in query.iter_mut() {
                    match selected {
                        Selected::Selected { .. } => {
                            entities.push(entity);
                        }
                        Selected::Unselected => {}
                    }
                }
                if entities.is_empty() {
                    return;
                }
                for i in 0..entities.len() - 1 {
                    events.send(SpawnElasticEvent {
                        elastic: Elastic((entities[i], entities[i + 1])),
                        team: config.player_team,
                    });
                }
                break;
            }
        }
    }
    pub fn update(
        mut commands: Commands,
        mut elastic_query: Query<(Entity, &Elastic, &mut Transform)>,
        object_query: Query<(Entity, &Position)>,
        mut accel_query: Query<&mut Acceleration>,
        mut attachments: Query<&mut AttachedTo>,
    ) {
        for (entity, elastic, mut transform) in elastic_query.iter_mut() {
            if let (Ok((entity1, position1)), Ok((entity2, position2))) = (
                object_query.get(elastic.first()),
                object_query.get(elastic.second()),
            ) {
                let delta = position2.0 - position1.0;
                let direction = delta.normalize_or_zero();
                let magnitude = delta.length();
                let force = magnitude * magnitude * 0.0001;
                *accel_query.get_mut(entity1).unwrap() += Acceleration(direction * force);
                *accel_query.get_mut(entity2).unwrap() -= Acceleration(direction * force);

                // Set transform.
                *transform =
                    Self::get_transform(position1.0, position2.0, magnitude, zindex::ZOOIDS_MIN);
            } else {
                // Clean up invalid connections.
                commands.entity(entity).despawn();
                if let Ok(mut attached_to) = attachments.get_mut(elastic.first()) {
                    attached_to.retain(|&mut x| x != elastic.second())
                }
                if let Ok(mut attached_to) = attachments.get_mut(elastic.first()) {
                    attached_to.retain(|&mut x| x != elastic.second())
                }
            }
        }
    }
}
