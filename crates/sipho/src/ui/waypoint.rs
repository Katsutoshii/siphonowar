use crate::prelude::*;
use bevy::{prelude::*, utils::hashbrown::HashSet};

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
        attachments: Query<&AttachedTo>,
        mut objectives: Query<&mut Objectives>,
        assets: Res<WaypointAssets>,
        // meshes: Res<Assets<Mesh>>,
    ) {
        for control in control_events.read() {
            if !(control.is_pressed(ControlAction::Move)
                || control.is_pressed(ControlAction::AttackMove))
            {
                continue;
            }

            // Spawn a new waypoint.
            let waypoint_bundle = Waypoint::default().bundle(
                &assets,
                control.position.extend(zindex::WAYPOINT),
                control.mode,
            );
            let waypoint_entity = commands.spawn(waypoint_bundle).id();

            for (entity, selected, object, attached_to) in selection.iter() {
                if selected.is_selected() {
                    // When a head is selected, clear the objectives of all attached children.
                    if *object == Object::Head && !attached_to.is_empty() {
                        // DFS to clear objectives of all attached.
                        let mut visited: HashSet<Entity> = HashSet::new();
                        let mut stack: Vec<Entity> = Vec::new();
                        stack.extend(attached_to.iter());
                        while let Some(entity) = stack.pop() {
                            if !visited.insert(entity) {
                                continue;
                            }

                            // Clear objective.
                            if let Ok(mut objectives) = objectives.get_mut(entity) {
                                objectives.clear();
                            }

                            if let Ok(attached_to) = attachments.get(entity) {
                                stack.extend(attached_to.iter());
                            }
                        }
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

    pub fn bundle(
        self,
        assets: &WaypointAssets,
        translation: Vec3,
        mode: ControlMode,
    ) -> impl Bundle {
        (
            Name::new("Waypoint"),
            PbrBundle {
                mesh: assets.mesh.clone(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(self.size).extend(1.))
                    // .with_rotation(Quat::from_axis_angle(Vec3::Y, PI / 2.))
                    .with_translation(translation),
                material: match mode {
                    ControlMode::Normal => assets.blue_material.clone(),
                    ControlMode::Attack => assets.red_material.clone(),
                },
                ..default()
            },
            CarriedBy::default(),
            Velocity::ZERO,
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
