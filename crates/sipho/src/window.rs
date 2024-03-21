use bevy::{
    prelude::*,
    window::{Cursor, PresentMode, PrimaryWindow, WindowMode, WindowTheme},
};

use crate::prelude::Configs;

pub trait ScalableWindow {
    fn scaled_size(&self) -> Vec2;
}
impl ScalableWindow for Window {
    fn scaled_size(&self) -> Vec2 {
        Vec2 {
            x: self.width(),
            y: self.height(),
            // x: self.physical_width() as f32 * self.scale_factor(),
            // y: self.physical_height() as f32 * self.scale_factor(),
        }
    }
}
pub fn custom_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            cursor: Cursor {
                visible: true,
                ..default()
            },
            title: "Bevy Zooids".into(),
            present_mode: PresentMode::AutoVsync,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            window_theme: Some(WindowTheme::Dark),
            enabled_buttons: bevy::window::EnabledButtons {
                maximize: false,
                ..Default::default()
            },
            visible: true,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        }),
        ..default()
    }
}

pub fn resize_window(mut query: Query<&mut Window, With<PrimaryWindow>>, configs: Res<Configs>) {
    if configs.is_changed() {
        let mut window = query.single_mut();
        let scale_factor = window.scale_factor();
        if configs.window_size != Vec2::ZERO {
            window.resolution.set_physical_resolution(
                (configs.window_size.x * scale_factor) as u32,
                (configs.window_size.y * scale_factor) as u32,
            );
        }
    }
}
