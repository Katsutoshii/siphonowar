pub mod controller;

use std::f32::consts::PI;

use bevy::prelude::*;
pub use controller::{CameraController, CameraMoveEvent};

/// Marks the main camera.
#[derive(Component)]
pub struct MainCamera;
impl MainCamera {
    pub const THETA: f32 = PI / 8.;
    pub const FOV: f32 = PI / 4.;
}
impl MainCamera {
    pub fn y_offset(z: f32) -> f32 {
        Self::THETA.tan() * z
    }
}

pub trait CameraAspectRatio {
    fn aspect_ratio(&self) -> Vec2;
}
impl CameraAspectRatio for Camera {
    fn aspect_ratio(&self) -> Vec2 {
        let viewport_size = self.logical_viewport_size().unwrap();
        viewport_size / viewport_size.y
    }
}
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(controller::CameraControllerPlugin);
    }
}
