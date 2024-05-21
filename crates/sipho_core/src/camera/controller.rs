use bevy::{
    input::{mouse::MouseWheel, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};

use crate::prelude::*;

pub trait CameraAspectRatio {
    fn aspect_ratio(&self) -> Vec2;
}
impl CameraAspectRatio for Camera {
    fn aspect_ratio(&self) -> Vec2 {
        let viewport_size = self.logical_viewport_size().unwrap();
        viewport_size / viewport_size.y
    }
}

pub struct CameraControllerPlugin;
impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CameraMoveEvent>().add_systems(
            Update,
            (
                CameraController::update_bounds,
                CameraController::update_screen_control,
                CameraController::update_control,
            )
                .chain(),
        );
    }
}

#[derive(Event)]
pub struct CameraMoveEvent {
    pub position: Vec3,
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
        let height = camera_transform.translation.z;
        camera_transform.translation = position.extend(height);
        self.world2d_bounds
            .clamp3(&mut camera_transform.translation)
    }

    fn update_bounds(
        grid_spec: Res<GridSpec>,
        mut controller_query: Query<(&mut Self, &GlobalTransform)>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        if !grid_spec.is_changed() {
            return;
        }
        if let Ok((mut controller, camera_transform)) = controller_query.get_single_mut() {
            if let Some(world2d_size) = Self::get_world2d_size(camera_transform, window.single()) {
                controller.world2d_bounds = grid_spec.world2d_bounds();
                controller.world2d_bounds.min += world2d_size;
                controller.world2d_bounds.max -= world2d_size;
            }
        }
    }

    fn get_world2d_size(camera_transform: &GlobalTransform, window: &Window) -> Option<Vec2> {
        Some(
            1.5 * Vec2 {
                x: 1.,
                y: window.height() / window.width(),
            } * camera_transform.translation().z
                * (MainCamera::FOV / 2.).tan(),
        )
    }

    pub fn update_control(
        mut controller_query: Query<(&mut Self, &mut Transform)>,
        mut controls: EventReader<ControlEvent>,
        mut event_writer: EventWriter<CameraMoveEvent>,
        mut mouse_wheel: EventReader<MouseWheel>,
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
                        position: camera_transform.translation,
                    });
                }
                _ => {}
            }
        }
        for event in mouse_wheel.read() {
            let height = camera_transform.translation.z;
            camera_transform.translation.z =
                (height - event.y * 50.).clamp(zindex::MIN_CAMERA, zindex::CAMERA);
            event_writer.send(CameraMoveEvent {
                position: camera_transform.translation,
            });
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

        let mut force = Vec2::ZERO;
        controller.velocity = Vec2::ZERO;
        let window_size = window.scaled_size();

        if let Some(centered_cursor_position) = window.cursor_position() {
            let boundary = 2.;
            // Screen border panning.
            force += if centered_cursor_position.x < boundary {
                -Vec2::X
            } else if centered_cursor_position.x > window_size.x - boundary {
                Vec2::X
            } else {
                Vec2::ZERO
            };
            force += if centered_cursor_position.y < boundary {
                Vec2::Y
            } else if centered_cursor_position.y > window_size.y - boundary {
                -Vec2::Y
            } else {
                Vec2::ZERO
            };

            controller.velocity += force;
            let position = camera_transform.translation.xy()
                + dt * controller.velocity * controller.sensitivity;
            controller.set_position(&mut camera_transform, position);
            event_writer.send(CameraMoveEvent {
                position: camera_transform.translation,
            });
        }
    }
}
