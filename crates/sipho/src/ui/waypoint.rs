use crate::prelude::*;
use bevy::{prelude::*, utils::hashbrown::HashSet};

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct WaypointPlugin;
impl Plugin for WaypointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointAssets>().add_systems(
            FixedUpdate,
            (
                Waypoint::cleanup.in_set(FixedUpdateStage::Cleanup),
                Waypoint::update.in_set(FixedUpdateStage::Spawn),
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
                for objective in objectives.iter() {
                    if let Some(entity) = objective.get_followed_entity() {
                        followed_entities.insert(entity);
                    }
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
        selection: Query<(Entity, &Selected, &Object, &AttachedTo)>,
        mut commands: Commands,
        mut objectives: Query<&mut Objectives>,
        assets: Res<WaypointAssets>,
    ) {
        for control in control_events.read() {
            if !(control.is_pressed(ControlAction::Move)
                || control.is_pressed(ControlAction::AttackMove))
            {
                continue;
            }

            // Spawn a new waypoint.
            let waypoint_bundle =
                Waypoint::default().bundle(&assets, control.position, control.mode);
            let waypoint_entity = commands.spawn(waypoint_bundle).id();

            for (entity, selected, object, attached_to) in selection.iter() {
                if selected.is_selected() {
                    // Don't change objectives for workers that are in the middle of the parent.
                    if *object == Object::Worker && attached_to.len() > 1 {
                        continue;
                    }
                    let mut objectives = objectives.get_mut(entity).unwrap();
                    objectives.clear();
                    let objective = match control.mode {
                        ControlMode::Normal => Objective::FollowEntity(waypoint_entity),
                        ControlMode::Attack => Objective::AttackFollowEntity(waypoint_entity),
                    };
                    objectives.push(objective);
                }
            }
        }
    }

    pub fn bundle(self, assets: &WaypointAssets, position: Vec2, mode: ControlMode) -> impl Bundle {
        (
            Name::new("Waypoint"),
            PbrBundle {
                mesh: assets.mesh.clone(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(self.size).extend(1.))
                    // .with_rotation(Quat::from_axis_angle(Vec3::Y, PI / 2.))
                    .with_translation(position.extend(zindex::WAYPOINT)),
                material: match mode {
                    ControlMode::Normal => assets.blue_material.clone(),
                    ControlMode::Attack => assets.red_material.clone(),
                },
                ..default()
            },
            CarriedBy::default(),
            PhysicsBundle {
                position: Position(position),
                ..default()
            },
            self,
        )
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct WaypointAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<StandardMaterial>,
    pub red_material: Handle<StandardMaterial>,
}
impl FromWorld for WaypointAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: {
                // let asset_server = world.get_resource::<AssetServer>().unwrap();
                // let mesh = asset_server.load("models/triangle/triangle.gltf#Mesh0/Primitive0");
                // mesh
                let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
                meshes.add(Mesh::from(RegularPolygon {
                    circumcircle: Circle {
                        radius: 2f32.sqrt() / 2.,
                    },
                    sides: 3,
                }))
            },
            blue_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial::from(Color::TURQUOISE.with_a(0.5)))
            },
            red_material: {
                let mut materials = world
                    .get_resource_mut::<Assets<StandardMaterial>>()
                    .unwrap();
                materials.add(StandardMaterial::from(Color::TOMATO.with_a(0.5)))
            },
        }
    }
}
