use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*, sprite::MaterialMesh2dBundle};

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
    pub zindex: f32,
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
    pub material_mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub selected: Selected,
    pub health: Health,
    pub neighbors: NeighborsBundle,
    pub name: Name,
}
impl ObjectBundle {
    pub fn new(config: &ObjectConfig, spec: ObjectSpec) -> Self {
        Self {
            object: spec.object,
            team: spec.team,
            objectives: spec.objectives,
            physics: PhysicsBundle {
                material: config.physics_material,
                velocity: spec
                    .velocity
                    .unwrap_or(Velocity(Vec2::ONE) * config.spawn_velocity),
                ..default()
            },
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
}
impl ObjectCommands<'_, '_> {
    pub fn spawn(&mut self, spec: ObjectSpec) {
        let config = &self.configs[&spec.object];
        let team_material = self.assets.get_team_material(spec.team);
        let background = self.background_bundle(
            team_material.clone(),
            match spec.object {
                Object::Worker | Object::Head => zindex::ZOOID_HEAD_BACKGROUND,
                Object::Plankton | Object::Food => zindex::PLANKTON_BACKGROUND,
            },
        );
        match spec.object {
            Object::Worker => {
                self.commands
                    .spawn((
                        ZooidWorker::default(),
                        NearestZooidHead::default(),
                        ObjectBundle {
                            material_mesh: MaterialMesh2dBundle::<ColorMaterial> {
                                mesh: self.assets.mesh.clone().into(),
                                transform: Transform::default()
                                    .with_scale(Vec2::splat(10.0).extend(1.))
                                    .with_translation(spec.position.extend(spec.zindex)),
                                material: team_material.primary,
                                ..default()
                            },
                            health: Health::new(3),
                            name: Name::new("Zooid"),
                            ..ObjectBundle::new(config, spec)
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(background);
                    });
            }
            Object::Head => {
                let position = spec.position;
                let mut entity_commands = self.commands.spawn((
                    ZooidHead,
                    Consumer::default(),
                    ObjectBundle {
                        material_mesh: MaterialMesh2dBundle::<ColorMaterial> {
                            mesh: self.assets.mesh.clone().into(),
                            transform: Transform::default()
                                .with_scale(Vec2::splat(20.0).extend(1.))
                                .with_translation(position.extend(zindex::ZOOID_HEAD)),
                            material: team_material.primary,
                            ..default()
                        },
                        health: Health::new(3),
                        name: Name::new("ZooidHead"),
                        ..ObjectBundle::new(config, spec)
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
                            team: Team::None,
                            material_mesh: MaterialMesh2dBundle::<ColorMaterial> {
                                mesh: self.assets.mesh.clone().into(),
                                transform: Transform::default()
                                    .with_scale(Vec2::splat(10.0).extend(1.))
                                    .with_translation(spec.position.extend(zindex::PLANKTON)),
                                material: team_material.primary,
                                ..default()
                            },
                            health: Health::new(1),
                            name: Name::new("Plankton"),
                            ..ObjectBundle::new(config, spec)
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(background);
                    });
            }
            Object::Food => {
                self.commands.spawn(ObjectBundle {
                    team: Team::None,
                    material_mesh: MaterialMesh2dBundle::<ColorMaterial> {
                        mesh: self.assets.mesh.clone().into(),
                        transform: Transform::default()
                            .with_scale(Vec2::splat(10.0).extend(1.))
                            .with_translation(spec.position.extend(zindex::FOOD)),
                        material: team_material.secondary,
                        ..default()
                    },
                    health: Health::new(1),
                    name: Name::new("Food"),
                    ..ObjectBundle::new(config, spec)
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
        self.commands.entity(entity).despawn_recursive();
    }

    pub fn background_bundle(&self, team_material: TeamMaterials, zindex: f32) -> impl Bundle {
        (
            ObjectBackground,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: self.assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(1.5).extend(1.))
                    .with_translation(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: zindex,
                    }),
                material: team_material.background,
                ..default()
            },
            Name::new("ObjectBackground"),
        )
    }
}
