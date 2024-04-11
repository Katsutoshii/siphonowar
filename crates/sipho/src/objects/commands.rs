use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};

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
    pub selected: Selected,
    pub health: Health,
    pub neighbors: NeighborsBundle,
    pub attached_to: AttachedTo,
    pub carried_by: CarriedBy,
    pub name: Name,
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
                material: config.physics_material,
                velocity: spec
                    .velocity
                    .unwrap_or(Velocity(Vec2::ONE) * config.spawn_velocity),
                ..default()
            },
            health: Health::new(config.health),
            name: Name::new(name),
            ..default()
        }
    }
}

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct ObjectCommands<'w, 's> {
    assets: ResMut<'w, ObjectAssets>,
    commands: Commands<'w, 's>,
    configs: Res<'w, ObjectConfigs>,
    parents: Query<'w, 's, &'static Children, Without<Parent>>,
    #[allow(clippy::type_complexity)]
    children: Query<'w, 's, &'static Object, With<Parent>>,
    despawn_events: EventWriter<'w, DespawnEvent>,
    time: Res<'w, Time>,
}
impl ObjectCommands<'_, '_> {
    pub fn spawn(&mut self, spec: ObjectSpec) {
        let config = &self.configs[&spec.object];
        let team_material = self.assets.get_team_material(spec.team);
        let background = self.background_bundle(team_material.clone());
        match spec.object {
            Object::Worker => {
                self.commands
                    .spawn((
                        ZooidWorker::default(),
                        NearestZooidHead::default(),
                        ObjectBundle {
                            mesh: self.assets.mesh.clone(),
                            material: team_material.primary,
                            ..ObjectBundle::new(config, spec, &self.time)
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(background);
                    });
            }
            Object::Head => {
                let mut entity_commands = self.commands.spawn((
                    ZooidHead,
                    Consumer::new(3),
                    ObjectBundle {
                        mesh: self.assets.mesh.clone(),
                        material: team_material.primary,
                        ..ObjectBundle::new(config, spec, &self.time)
                    },
                ));
                entity_commands.with_children(|parent| {
                    parent.spawn(background);
                });
                let entity = entity_commands.id();
                entity_commands.insert(Objectives::new(Objective::FollowEntity(entity)));
            }
            Object::Plankton => {
                self.commands
                    .spawn((
                        Plankton,
                        ObjectBundle {
                            mesh: self.assets.mesh.clone(),
                            material: team_material.primary,
                            ..ObjectBundle::new(config, spec, &self.time)
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(background);
                    });
            }
            Object::Food => {
                self.commands.spawn(ObjectBundle {
                    mesh: self.assets.mesh.clone(),
                    material: team_material.secondary,
                    ..ObjectBundle::new(config, spec, &self.time)
                });
            }
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        if let Ok(children) = self.parents.get(entity) {
            for &child in children {
                if self.children.get(child).is_ok() {
                    self.commands.entity(child).remove_parent_in_place();
                }
            }
        }
        self.despawn_events.send(DespawnEvent(entity));
    }

    pub fn background_bundle(&self, team_material: TeamMaterials) -> impl Bundle {
        (
            ObjectBackground,
            PbrBundle {
                mesh: self.assets.mesh.clone(),
                transform: Transform::default().with_scale(Vec3::splat(1.5)),
                material: team_material.background,
                ..default()
            },
            Name::new("ObjectBackground"),
        )
    }
}
