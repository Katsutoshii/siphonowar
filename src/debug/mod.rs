use sipho::prelude::*;

pub mod console;
pub mod fps;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        app.add_plugins((
            WorldInspectorPlugin::default().run_if(in_state(GameState::Paused)),
            console::CustomConsolePlugin,
            fps::FpsPlugin,
        ));
    }
}
