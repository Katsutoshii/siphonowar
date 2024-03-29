use std::f32::consts::PI;

use crate::prelude::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::hashbrown::HashSet};

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct WaypointPlugin;
impl Plugin for WaypointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointAssets>().add_systems(
            FixedUpdate,
            (
                Waypoint::cleanup.in_set(SystemStage::Despawn),
                Waypoint::update.in_set(SystemStage::Spawn),
            )
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Component, Debug)]
pub struct Waypoint {
    pub active: bool,
    pub size: f32,
}
impl Default for Waypoint {
    fn default() -> Self {
        Self {
            active: false,
            size: 10.0,
        }
    }
}
impl Waypoint {
    /// Waypoint cleanup must happen one frame before update.
    pub fn cleanup(
        all_objectives: Query<&Objectives, Without<Waypoint>>,
        waypoints: Query<Entity, With<Waypoint>>,
        mut commands: Commands,
        mut input_actions: EventReader<ControlEvent>,
    ) {
        for &ControlEvent { action, .. } in input_actions.read() {
            if let ControlAction::Move | ControlAction::AttackMove = action {
            } else {
                continue;
            }

            let mut followed_entities = HashSet::new();
            for objectives in all_objectives.iter() {
                if let Some(entity) = objectives.last().get_followed_entity() {
                    followed_entities.insert(entity);
                }
            }
            for entity in waypoints.iter() {
                if !followed_entities.contains(&entity) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    pub fn update(
        mut control_events: EventReader<ControlEvent>,
        mut selection: Query<(&Selected, &mut Objectives), Without<Self>>,
        mut commands: Commands,
        assets: Res<WaypointAssets>,
    ) {
        for control in control_events.read() {
            if !(control.is_pressed(ControlAction::Move)
                || control.is_pressed(ControlAction::AttackMove))
            {
                continue;
            }

            if control.action == ControlAction::AttackMove && control.mode != ControlMode::Attack {
                error!("WTF");
            }
            // Spawn a new waypoint.
            let waypoint_bundle = Waypoint::default().bundle(
                &assets,
                control.position.extend(zindex::WAYPOINT),
                control.mode,
            );
            let entity = commands.spawn(waypoint_bundle).id();

            for (selected, mut objectives) in selection.iter_mut() {
                if selected.is_selected() {
                    match objectives.last() {
                        Objective::AttackEntity(_) => {
                            continue;
                        }
                        _ => {
                            objectives.clear();
                            objectives.push(Objective::FollowEntity(entity));
                        }
                    }
                }
            }
        }
    }

    pub fn bundle(
        self,
        assets: &WaypointAssets,
        translation: Vec3,
        mode: ControlMode,
    ) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(self.size).extend(1.))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, PI))
                    .with_translation(translation),
                material: match mode {
                    ControlMode::Normal => assets.blue_material.clone(),
                    ControlMode::Attack => assets.red_material.clone(),
                },
                ..default()
            },
            Velocity::ZERO,
            self,
        )
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct WaypointAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<ColorMaterial>,
    pub red_material: Handle<ColorMaterial>,
}
impl FromWorld for WaypointAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: {
                let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                meshes.add(Mesh::from(RegularPolygon {
                    circumcircle: Circle {
                        radius: 2f32.sqrt() / 2.,
                    },
                    sides: 3,
                }))
            },
            blue_material: {
                let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
                materials.add(ColorMaterial::from(Color::TURQUOISE.with_a(0.5)))
            },
            red_material: {
                let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
                materials.add(ColorMaterial::from(Color::TOMATO.with_a(0.5)))
            },
        }
    }
}
