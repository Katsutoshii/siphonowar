use std::f32::consts::PI;

use crate::{grid::CreateWaypointEvent, prelude::*};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::hashbrown::HashSet};

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct WaypointPlugin;
impl Plugin for WaypointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointAssets>().add_systems(
            FixedUpdate,
            (
                Waypoint::update.in_set(SystemStage::PostApply),
                Waypoint::cleanup
                    .in_set(SystemStage::PostApply)
                    .after(Waypoint::update),
            ),
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
    pub fn cleanup(
        all_objectives: Query<&Objectives, Without<Waypoint>>,
        waypoints: Query<Entity, With<Waypoint>>,
        mut commands: Commands,
        mut input_actions: EventReader<ControlEvent>,
    ) {
        for &ControlEvent {
            action,
            state: _,
            position: _,
        } in input_actions.read()
        {
            if action != ControlAction::Move {
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
        mut selection: Query<(&Selected, &mut Objectives, &Transform), Without<Self>>,
        mut event_writer: EventWriter<CreateWaypointEvent>,
        mut commands: Commands,
        assets: Res<WaypointAssets>,
    ) {
        for control in control_events.read() {
            if !control.is_pressed(ControlAction::Move) {
                continue;
            }

            // Spawn a new waypoint.
            let waypoint_bundle =
                Waypoint::default().bundle(&assets, control.position.extend(zindex::WAYPOINT));
            let entity = commands.spawn(waypoint_bundle).id();

            let mut sources = Vec::new();
            for (selected, mut objectives, transform) in selection.iter_mut() {
                if selected.is_selected() {
                    objectives.clear();
                    objectives.push(Objective::FollowEntity(entity));
                    sources.push(transform.translation.xy());
                }
            }
            if !sources.is_empty() {
                event_writer.send(CreateWaypointEvent {
                    sources,
                    destination: control.position,
                });
            }
        }
    }

    pub fn bundle(self, assets: &WaypointAssets, translation: Vec3) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(self.size).extend(1.))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, PI))
                    .with_translation(translation),
                material: assets.blue_material.clone(),
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
}
impl FromWorld for WaypointAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(RegularPolygon {
                circumcircle: Circle {
                    radius: 2f32.sqrt() / 2.,
                },
                sides: 3,
            }))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            blue_material: materials.add(ColorMaterial::from(Color::TURQUOISE.with_a(0.5))),
        }
    }
}
