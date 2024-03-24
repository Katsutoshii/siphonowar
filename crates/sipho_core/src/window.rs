use bevy::{
    prelude::*,
    window::{Cursor, PresentMode, WindowMode, WindowTheme},
};

pub trait ScalableWindow {
    fn scaled_size(&self) -> Vec2;
}
impl ScalableWindow for Window {
    fn scaled_size(&self) -> Vec2 {
        Vec2 {
            x: self.width(),
            y: self.height(),
        }
    }
}
pub fn custom_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            cursor: Cursor {
                visible: false,
                ..default()
            },
            title: "Siphonowar".into(),
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
