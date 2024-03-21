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

/// System param to allow spawning effects.
#[derive(SystemParam)]
pub struct ObjectCommands<'w, 's> {
    assets: ResMut<'w, ObjectAssets>,
    commands: Commands<'w, 's>,
    configs: Res<'w, Configs>,
    event_writer: EventWriter<'w, CreateWaypointEvent>,
}
impl ObjectCommands<'_, '_> {
    pub fn spawn(&mut self, spec: ObjectSpec) {
        let config = &self.configs.objects[&spec.object];
        let team_material = self.assets.get_team_material(spec.team);
        let velocity = if let Some(velocity) = spec.velocity {
            velocity
        } else {
            Velocity(Vec2::ONE) * config.spawn_velocity
        };
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
                        Object::Worker,
                        spec.team,
                        PhysicsBundle {
                            material: config.physics_material,
                            velocity,
                            ..default()
                        },
                        spec.objectives,
                        MaterialMesh2dBundle::<ColorMaterial> {
                            mesh: self.assets.mesh.clone().into(),
                            transform: Transform::default()
                                .with_scale(Vec2::splat(10.0).extend(1.))
                                .with_translation(spec.position.extend(spec.zindex)),
                            material: team_material.primary,
                            ..default()
                        },
                        Selected::default(),
                        Health::new(3),
                        NeighborsBundle::default(),
                        Name::new("Zooid"),
                    ))
                    .with_children(|parent| {
                        parent.spawn(background);
                    });
            }
            Object::Head => {
                let mut entity_commands = self.commands.spawn((
                    ZooidHead,
                    Object::Head,
                    spec.team,
                    MaterialMesh2dBundle::<ColorMaterial> {
                        mesh: self.assets.mesh.clone().into(),
                        transform: Transform::default()
                            .with_scale(Vec2::splat(20.0).extend(1.))
                            .with_translation(spec.position.extend(zindex::ZOOID_HEAD)),
                        material: team_material.primary,
                        ..default()
                    },
                    PhysicsBundle {
                        material: config.physics_material,
                        velocity,
                        ..default()
                    },
                    spec.objectives,
                    Selected::default(),
                    Health::new(6),
                    NeighborsBundle::default(),
                    Name::new("ZooidHead"),
                ));
                entity_commands.with_children(|parent| {
                    parent.spawn(background);
                });
                let entity = entity_commands.id();
                entity_commands.insert(Objectives::new(Objective::FollowEntity(entity)));
                self.event_writer.send(CreateWaypointEvent {
                    destination: spec.position,
                    sources: vec![spec.position],
                });
            }
            Object::Plankton => {
                self.commands
                    .spawn((
                        Plankton,
                        Object::Plankton,
                        Team::None,
                        MaterialMesh2dBundle::<ColorMaterial> {
                            mesh: self.assets.mesh.clone().into(),
                            transform: Transform::default()
                                .with_scale(Vec2::splat(10.0).extend(1.))
                                .with_translation(spec.position.extend(zindex::PLANKTON)),
                            material: team_material.primary,
                            ..default()
                        },
                        PhysicsBundle {
                            material: config.physics_material,
                            velocity,
                            ..default()
                        },
                        spec.objectives,
                        Health::new(1),
                        Selected::default(),
                        NeighborsBundle::default(),
                        Name::new("Plankton"),
                    ))
                    .with_children(|parent| {
                        parent.spawn(background);
                    });
            }
            Object::Food => {
                self.commands.spawn((
                    Object::Food,
                    Team::None,
                    MaterialMesh2dBundle::<ColorMaterial> {
                        mesh: self.assets.mesh.clone().into(),
                        transform: Transform::default()
                            .with_scale(Vec2::splat(10.0).extend(1.))
                            .with_translation(spec.position.extend(zindex::FOOD)),
                        material: team_material.secondary,
                        ..default()
                    },
                    PhysicsBundle {
                        material: config.physics_material,
                        velocity: Velocity::ZERO,
                        ..default()
                    },
                    spec.objectives,
                    Health::new(1),
                    Selected::default(),
                    NeighborsBundle::default(),
                    Name::new("Food"),
                ));
            }
        }
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
        )
    }
}
