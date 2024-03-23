use sipho::prelude::*;

pub mod console;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        use bevy::diagnostic::{
            // FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin,
        };
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        app.add_plugins((
            WorldInspectorPlugin::default().run_if(in_state(PausedState::Paused)),
            console::CustomConsolePlugin,
            // FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ));
    }
}
