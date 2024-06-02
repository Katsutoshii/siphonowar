use crate::{
    prelude::*,
    ui::selector::{HighlightBundle, SelectorAssets},
};
use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use super::{
    neighbors::NeighborsBundle,
    object::ObjectBackground,
    plankton::Plankton,
    zooid_head::{NearestZooidHead, ZooidHead},
    zooid_worker::ZooidWorker,
    ObjectAssets, TeamMaterials,
};

#[derive(Default, Debug)]
pub struct ObjectSpec {
    pub object: Object,
    pub position: Vec2,
    pub team: Team,
    pub velocity: Option<Velocity>,
    pub objectives: Objectives,
    pub selected: bool,
}

#[derive(Bundle, Default)]
pub struct ObjectBundle {
    pub object: Object,
    pub team: Team,
    pub physics: PhysicsBundle,
    pub objectives: Objectives,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub health: Health,
    pub neighbors: NeighborsBundle,
    pub attached_to: AttachedTo,
    pub path: PathToHead,
    pub carried_by: CarriedBy,
    pub grid_raycast_target: GridRaycastTarget,
    pub name: Name,
    pub fog_entity: FogEntity,
    pub selectable: Selectable,
}
impl ObjectBundle {
    /// Returns random value [0, 1.)
    fn random_offset(time: &Time) -> f32 {
        time.elapsed().as_secs_f32() / time.wrap_period().as_secs_f32()
    }
    pub fn new(config: &ObjectConfig, spec: ObjectSpec, time: &Time) -> Self {
        let name: &'static str = spec.object.into();
        Self {
            object: spec.object,
            team: spec.team,
            objectives: spec.objectives,
            transform: Transform {
                scale: Vec3::splat(config.radius),
                translation: spec
                    .position
                    .extend(spec.object.zindex() + 0.1 * Self::random_offset(time)),
                ..default()
            },
            physics: PhysicsBundle {
                material: config.physics_material.clone(),
                position: Position(spec.position),
                velocity: spec
                    .velocity
                    .unwrap_or(Velocity(Vec2::ONE) * config.spawn_velocity),
                mass: Mass(1.0),
                ..default()
            },
            health: Health::new(config.health),
            name: Name::new(name),
            neighbors: NeighborsBundle {
                grid_entity: GridEntity {
                    publish_events: true,
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }
}

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct ObjectCommands<'w, 's> {
    assets: Res<'w, ObjectAssets>,
    selector_assets: Res<'w, SelectorAssets>,
    commands: Commands<'w, 's>,
    configs: Res<'w, ObjectConfigs>,
    parents: Query<'w, 's, &'static Children, Without<Parent>>,
    children: Query<'w, 's, &'static Object, With<Parent>>,
    despawn_events: EventWriter<'w, DespawnEvent>,
    time: Res<'w, Time>,
    obstacles: Res<'w, Grid2<Obstacle>>,
}
impl ObjectCommands<'_, '_> {
    pub fn spawn(&mut self, spec: ObjectSpec) -> Option<EntityCommands> {
        let config = &self.configs[&spec.object];
        let team_material = self.assets.get_team_material(spec.team);
        if let Some(rowcol) = self.obstacles.to_rowcol(spec.position) {
            if self.obstacles[rowcol] != Obstacle::Empty {
                return None;
            }
        }
        let commands = match spec.object {
            Object::Worker => {
                let mesh = self.assets.object_meshes[&Object::Worker].clone();
                let background = self.background_bundle(team_material.clone(), mesh.clone());
                let mut commands = self.commands.spawn((
                    ZooidWorker::default(),
                    NearestZooidHead::default(),
                    ObjectBundle {
                        mesh: mesh.clone(),
                        material: team_material.primary,
                        ..ObjectBundle::new(config, spec, &self.time)
                    },
                ));
                commands.with_children(|parent| {
                    parent.spawn(background);
                });
                commands
            }
            Object::Shocker => {
                let mesh = self.assets.object_meshes[&Object::Shocker].clone();
                let background = self.background_bundle(team_material.clone(), mesh.clone());
                let mut commands = self.commands.spawn((
                    NearestZooidHead::default(),
                    ObjectBundle {
                        mesh,
                        material: team_material.primary,
                        ..ObjectBundle::new(config, spec, &self.time)
                    },
                ));
                commands.with_children(|parent| {
                    parent.spawn(background);
                });
                commands
            }
            Object::Armor => {
                let mesh = self.assets.object_meshes[&Object::Armor].clone();
                let background = self.background_bundle(team_material.clone(), mesh.clone());
                let mut commands = self.commands.spawn((
                    NearestZooidHead::default(),
                    ObjectBundle {
                        mesh,
                        material: team_material.primary,
                        ..ObjectBundle::new(config, spec, &self.time)
                    },
                ));
                commands.with_children(|parent| {
                    parent.spawn(background);
                });
                commands
            }
            Object::Head => {
                let selected = spec.selected;
                let mesh = self.assets.object_meshes[&Object::Head].clone();
                let background = self.background_bundle(team_material.clone(), mesh.clone());
                let mut commands = self.commands.spawn((
                    ZooidHead::default(),
                    Consumer::new(20),
                    ObjectBundle {
                        mesh: mesh.clone(),
                        material: team_material.primary,
                        ..ObjectBundle::new(config, spec, &self.time)
                    },
                ));
                commands.insert(PathToHead {
                    head: Some(commands.id()),
                    ..default()
                });
                if selected {
                    commands.insert(Selected);
                }
                commands.with_children(|parent| {
                    parent.spawn(background);

                    if selected {
                        parent.spawn(HighlightBundle::new(
                            mesh.clone(),
                            self.selector_assets.white_material.clone(),
                            1.5,
                        ));
                    }
                });
                let entity = commands.id();
                commands.insert(Objectives::new(Objective::FollowEntity(entity)));

                commands
            }
            Object::Plankton => {
                let mesh = self.assets.object_meshes[&Object::Plankton].clone();
                let background = self.background_bundle(team_material.clone(), mesh.clone());
                let mut commands = self.commands.spawn((
                    Plankton,
                    ObjectBundle {
                        mesh: mesh.clone(),
                        material: team_material.primary,
                        ..ObjectBundle::new(config, spec, &self.time)
                    },
                ));
                commands.with_children(|parent| {
                    parent.spawn(background);
                });
                commands
            }
            Object::Food => self.commands.spawn((
                PathToHeadFollower::default(),
                ObjectBundle {
                    mesh: self.assets.object_meshes[&Object::Food].clone(),
                    material: team_material.secondary,
                    ..ObjectBundle::new(config, spec, &self.time)
                },
            )),
            Object::BuilderPreview => unreachable!(),
        };
        Some(commands)
    }

    /// Queue a despawn event for this entity.
    pub fn deferred_despawn(&mut self, entity: Entity) {
        if let Ok(children) = self.parents.get(entity) {
            for &child in children {
                if self.children.get(child).is_ok() {
                    self.commands.entity(child).remove_parent_in_place();
                }
            }
        }
        self.despawn_events.send(DespawnEvent(entity));
    }

    pub fn background_bundle(
        &self,
        team_material: TeamMaterials,
        mesh: Handle<Mesh>,
    ) -> impl Bundle {
        (
            ObjectBackground,
            PbrBundle {
                mesh,
                transform: Transform {
                    scale: Vec3::splat(1.5),
                    translation: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: -0.1,
                    },
                    ..default()
                },
                material: team_material.background,
                ..default()
            },
            Name::new("ObjectBackground"),
        )
    }
}
