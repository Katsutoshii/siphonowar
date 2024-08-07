use crate::prelude::*;
use bevy::color::palettes::css::{TOMATO, TURQUOISE};
use bevy::{input::ButtonState, prelude::*, utils::hashbrown::HashSet};
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

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut control_events: EventReader<ControlEvent>,
        selection: Query<(Entity, &Object, &AttachedTo), With<Selected>>,
        teams: Query<&Team>,
        mut commands: Commands,
        mut objectives: Query<&mut Objectives>,
        assets: Res<WaypointAssets>,
        obstacles: Res<Grid2<Obstacle>>,
        team_config: Res<TeamConfig>,
    ) {
        for control in control_events.read() {
            if control.state != ButtonState::Pressed {
                continue;
            }
            let objective = match control.action {
                ControlAction::Move | ControlAction::AttackMove => {
                    // Don't spawn waypoints in obstacles.
                    let rowcol = obstacles.to_rowcol(control.position).unwrap();
                    if !obstacles.is_clear(rowcol) {
                        continue;
                    }

                    // Spawn a new waypoint.
                    let waypoint_bundle =
                        Waypoint::default().bundle(&assets, control.position, control.action);
                    let waypoint_entity = commands.spawn(waypoint_bundle).id();

                    Some(match control.action {
                        ControlAction::Move => Objective::FollowEntity(waypoint_entity),
                        ControlAction::AttackMove => Objective::AttackFollowEntity(waypoint_entity),
                        _ => unreachable!(),
                    })
                }
                ControlAction::Interact => {
                    if let Ok(team) = teams.get(control.entity) {
                        Some(if *team == team_config.player_team {
                            Objective::FollowEntity(control.entity)
                        } else {
                            Objective::AttackFollowEntity(control.entity)
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(objective) = objective {
                for (entity, object, attached_to) in selection.iter() {
                    let mut objectives = objectives.get_mut(entity).unwrap();
                    // Don't change objectives for workers that are in the middle of the parent.
                    if *object == Object::Worker && attached_to.len() > 1 {
                        if objectives.last() != &Objective::Idle {
                            objectives.clear();
                        }
                        continue;
                    }
                    objectives.clear();
                    objectives.push(objective.clone());
                }
            }
        }
    }

    pub fn bundle(
        self,
        assets: &WaypointAssets,
        position: Vec2,
        action: ControlAction,
    ) -> impl Bundle {
        (
            Name::new("Waypoint"),
            PbrBundle {
                mesh: assets.mesh.clone(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(self.size).extend(1.))
                    // .with_rotation(Quat::from_axis_angle(Vec3::Y, PI / 2.))
                    .with_translation(position.extend(zindex::WAYPOINT)),
                material: match action {
                    ControlAction::Move => assets.blue_material.clone(),
                    ControlAction::AttackMove => assets.red_material.clone(),
                    _ => unreachable!(),
                },
                ..default()
            },
            CarriedBy::default(),
            Position(position),
            // PhysicsBundle {
            //     position: Position(position),
            //     ..default()
            // },
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
            mesh: world.append_asset(Mesh::from(RegularPolygon {
                circumcircle: Circle {
                    radius: 2f32.sqrt() / 2.,
                },
                sides: 3,
            })),
            blue_material: world.append_asset(StandardMaterial::from(Color::from(
                TURQUOISE.with_alpha(0.5),
            ))),
            red_material: world
                .append_asset(StandardMaterial::from(Color::from(TOMATO.with_alpha(0.5)))),
        }
    }
}
