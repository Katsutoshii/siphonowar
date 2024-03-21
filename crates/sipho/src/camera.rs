use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::cursor::CursorAssets;
use crate::prelude::*;

use self::window::ScalableWindow;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .add_event::<CameraMoveEvent>()
            .add_systems(Startup, MainCamera::startup)
            .add_systems(
                FixedUpdate,
                (
                    CameraController::update_bounds.after(window::resize_window),
                    CameraController::update,
                    CameraController::update_drag,
                    CameraController::pan_to_position,
                ),
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
    pub fn startup(mut commands: Commands, assets: Res<CursorAssets>) {
        let camera_entity = commands
            .spawn((
                Camera2dBundle {
                    camera: Camera {
                        hdr: true, // 1. HDR is required for bloom
                        ..default()
                    },
                    tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
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
                }, // 3. Enable bloom for the camera
                CameraController::default(),
                InheritedVisibility::default(),
                MainCamera,
            ))
            .id();

        let cursor_bundle = Cursor.bundle(&assets, Vec2::ZERO.extend(zindex::CURSOR));
        let cursor_entity = commands.spawn(cursor_bundle).id();
        commands
            .entity(camera_entity)
            .push_children(&[cursor_entity]);
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
    fn update_bounds(
        grid_spec: Res<GridSpec>,
        configs: Res<Configs>,
        mut controller_query: Query<(&mut Self, &Camera, &GlobalTransform), With<MainCamera>>,
        window: Query<&Window, With<PrimaryWindow>>,
    ) {
        if !(grid_spec.is_changed() || configs.is_changed()) {
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

    pub fn update_drag(
        mut controller_query: Query<(&mut Self, &mut Transform), With<MainCamera>>,
        cursor: Query<&GlobalTransform, (With<Cursor>, Without<MainCamera>)>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        mut event_writer: EventWriter<CameraMoveEvent>,
    ) {
        let (mut controller, mut camera_transform) = controller_query.single_mut();
        let cursor = cursor.single();
        let cursor_position = cursor.translation().xy();

        // Middle mouse drag
        if mouse_input.pressed(MouseButton::Middle) {
            let delta = if let Some(last_drag_position) = controller.last_drag_position {
                let delta = last_drag_position - cursor_position;
                camera_transform.translation += delta.extend(0.);
                delta
            } else {
                Vec2::ZERO
            };
            controller.last_drag_position = Some(cursor_position + delta);
        } else if mouse_input.just_released(MouseButton::Middle) {
            controller.last_drag_position = None;
        }

        controller
            .world2d_bounds
            .clamp3(&mut camera_transform.translation);
        event_writer.send(CameraMoveEvent {
            position: camera_transform.translation.xy(),
        });
    }

    pub fn update(
        time: Res<Time>,
        mut controller_query: Query<(&mut Self, &mut Transform), With<MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mut event_writer: EventWriter<CameraMoveEvent>,
    ) {
        let dt = time.delta_seconds();
        let window = window_query.single();
        let (mut controller, mut camera_transform) = controller_query.single_mut();

        // let cursor = cursor.single();
        // let cursor_position = cursor.translation.xy();
        let mut acceleration = Vec2::ZERO;
        controller.velocity = Vec2::ZERO;
        let window_size = window.scaled_size();

        if let Some(centered_cursor_position) = window.cursor_position() {
            let boundary = 1.;
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
            camera_transform.translation +=
                controller.velocity.extend(0.) * dt * controller.sensitivity;
            controller
                .world2d_bounds
                .clamp3(&mut camera_transform.translation);
            event_writer.send(CameraMoveEvent {
                position: camera_transform.translation.xy(),
            });
        }
    }

    pub fn pan_to_position(
        mut control_events: EventReader<ControlEvent>,
        mut camera: Query<(&CameraController, &mut Transform), With<MainCamera>>,
    ) {
        for &ControlEvent {
            action,
            state: _,
            position,
        } in control_events.read()
        {
            if action != ControlAction::PanCamera {
                continue;
            }
            let (controller, mut camera_transform) = camera.single_mut();
            controller.set_position(&mut camera_transform, position);
        }
    }

    // Set camera position.
    pub fn set_position(&self, camera_transform: &mut Transform, position: Vec2) {
        camera_transform.translation = position.extend(0.);
        self.world2d_bounds
            .clamp3(&mut camera_transform.translation)
    }
}
