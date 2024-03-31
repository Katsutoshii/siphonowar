use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        tonemapping::Tonemapping,
    },
    input::ButtonState,
    prelude::*,
    window::PrimaryWindow,
};

use crate::prelude::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .add_event::<CameraMoveEvent>()
            .add_systems(Startup, MainCamera::startup)
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 500.,
            })
            .add_systems(
                Update,
                (
                    CameraController::update_bounds,
                    CameraController::update_screen_control,
                    CameraController::update_control,
                )
                    .chain()
                    .in_set(SystemStage::Spawn),
            );
    }
}

#[derive(Event)]
pub struct CameraMoveEvent {
    pub position: Vec2,
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;
impl MainCamera {
    pub fn startup(mut commands: Commands) {
        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true, // 1. HDR is required for bloom
                    ..default()
                },
                projection: PerspectiveProjection {
                    fov: PI / 2.0,
                    near: 0.1,
                    far: 2000.,
                    ..default()
                }
                .into(),
                transform: Transform::from_xyz(0.0, 0.0, zindex::CAMERA)
                    .with_rotation(Quat::from_axis_angle(Vec3::X, PI / 16.)),
                // .looking_at(Vec3::ZERO, Vec3::Z),
                tonemapping: Tonemapping::TonyMcMapface,
                ..default()
            },
            BloomSettings {
                intensity: 0.15 * 2.0,
                low_frequency_boost: 0.7,
                low_frequency_boost_curvature: 0.95,
                high_pass_frequency: 1.0,
                prefilter_settings: BloomPrefilterSettings {
                    threshold: 0.0,
                    threshold_softness: 0.0,
                },
                composite_mode: BloomCompositeMode::EnergyConserving,
            },
            CameraController::default(),
            InheritedVisibility::default(),
            MainCamera,
        ));
        commands.spawn(DirectionalLightBundle {
            transform: Transform::from_xyz(0.0, 0.0, zindex::CAMERA)
                .with_rotation(Quat::from_axis_angle(Vec3::X, PI / 8.)),
            directional_light: DirectionalLight {
                color: Color::ANTIQUE_WHITE,
                illuminance: 8000.,
                ..default()
            },
            ..default()
        });
    }
}

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub velocity: Vec2,
    pub last_drag_position: Option<Vec2>,
    world2d_bounds: Aabb2,
}
impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 1000.0,
            velocity: Vec2::ZERO,
            last_drag_position: None,
            world2d_bounds: Aabb2::default(),
        }
    }
}

impl CameraController {
    // Set camera position.
    pub fn set_position(&self, camera_transform: &mut Transform, position: Vec2) {
        camera_transform.translation = position.extend(zindex::CAMERA);
        self.world2d_bounds
            .clamp3(&mut camera_transform.translation)
    }

    fn update_bounds(
        grid_spec: Res<GridSpec>,
        mut controller_query: Query<(&mut Self, &Camera, &GlobalTransform), With<MainCamera>>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        if !grid_spec.is_changed() {
            return;
        }
        let (mut controller, camera, camera_transform) = controller_query.single_mut();
        if let Some(world2d_size) =
            Self::get_world2d_size(camera, camera_transform, window.single())
        {
            controller.world2d_bounds = grid_spec.world2d_bounds();
            controller.world2d_bounds.min += world2d_size * 0.5;
            controller.world2d_bounds.max -= world2d_size * 0.5;
        }
    }

    fn get_world2d_size(
        camera: &Camera,
        camera_transform: &GlobalTransform,
        window: &Window,
    ) -> Option<Vec2> {
        let camera_min = camera.viewport_to_world_2d(
            camera_transform,
            Vec2 {
                x: 0.,
                y: window.physical_height() as f32,
            },
        )?;
        let camera_max = camera.viewport_to_world_2d(
            camera_transform,
            Vec2 {
                x: window.physical_width() as f32,
                y: 0.,
            },
        )?;
        Some(camera_max - camera_min)
    }

    pub fn update_control(
        mut controller_query: Query<(&mut Self, &mut Transform), With<MainCamera>>,
        mut controls: EventReader<ControlEvent>,
        mut event_writer: EventWriter<CameraMoveEvent>,
    ) {
        let (mut controller, mut camera_transform) = controller_query.single_mut();
        for control in controls.read() {
            match control {
                ControlEvent {
                    action: ControlAction::DragCamera,
                    state: ButtonState::Pressed,
                    position,
                    ..
                } => {
                    let delta = if let Some(last_drag_position) = controller.last_drag_position {
                        let delta = last_drag_position - *position;
                        let new_position = camera_transform.translation.xy() + delta;
                        controller.set_position(&mut camera_transform, new_position);
                        delta
                    } else {
                        Vec2::ZERO
                    };
                    controller.last_drag_position = Some(*position + delta);
                }
                ControlEvent {
                    action: ControlAction::DragCamera,
                    state: ButtonState::Released,
                    ..
                } => {
                    controller.last_drag_position = None;
                }
                ControlEvent {
                    action: ControlAction::PanCamera,
                    state: ButtonState::Pressed,
                    position,
                    ..
                } => {
                    controller.set_position(&mut camera_transform, *position);
                    event_writer.send(CameraMoveEvent {
                        position: camera_transform.translation.xy(),
                    });
                }
                _ => {}
            }
        }
    }

    pub fn update_screen_control(
        time: Res<Time>,
        mut controller_query: Query<(&mut Self, &mut Transform), With<MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mut event_writer: EventWriter<CameraMoveEvent>,
    ) {
        let dt = time.delta_seconds();
        let window = window_query.single();
        let (mut controller, mut camera_transform) = controller_query.single_mut();

        let mut acceleration = Vec2::ZERO;
        controller.velocity = Vec2::ZERO;
        let window_size = window.scaled_size();

        if let Some(centered_cursor_position) = window.cursor_position() {
            let boundary = 2.;
            // Screen border panning.
            acceleration += if centered_cursor_position.x < boundary {
                -Vec2::X
            } else if centered_cursor_position.x > window_size.x - boundary {
                Vec2::X
            } else {
                Vec2::ZERO
            };
            acceleration += if centered_cursor_position.y < boundary {
                Vec2::Y
            } else if centered_cursor_position.y > window_size.y - boundary {
                -Vec2::Y
            } else {
                Vec2::ZERO
            };

            controller.velocity += acceleration;
            let position = camera_transform.translation.xy()
                + dt * controller.velocity * controller.sensitivity;
            controller.set_position(&mut camera_transform, position);
            event_writer.send(CameraMoveEvent {
                position: camera_transform.translation.xy(),
            });
        }
    }
}
