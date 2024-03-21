use bevy::prelude::*;

pub mod console;

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
        use bevy_inspector_egui::quick::WorldInspectorPlugin;

        app.add_plugins((
            WorldInspectorPlugin::default(),
            console::CustomConsolePlugin,
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, Self::startup);
    }
}
impl DebugPlugin {
    fn startup(mut commands: Commands) {
        commands.spawn((TextBundle::from_section(
            [
                "  Controls:",
                "    Create your spawner: 'x'",
                "    Move camera: move mouse to border",
                "    Move waypoint: right click",
                "    Spawn zooids: 'z'",
                "    Despawn zooids: 'd'",
                "    Save scene: 's'",
                "    Open editor: 'e'",
                "    -",
            ]
            .join("\n"),
            TextStyle {
                font_size: 18.0,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            ..default()
        }),));
    }
}
