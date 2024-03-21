use crate::{objects::ObjectSpec, prelude::*};
use bevy::prelude::*;
use bevy_console::{reply, AddConsoleCommand, ConsoleCommand, ConsolePlugin};
use clap::Parser;

/// Plugin for input action events.
pub struct CustomConsolePlugin;
impl Plugin for CustomConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin)
            .add_console_command::<SpawnCommand, _>(SpawnCommand::update);
    }
}

/// Example command
#[derive(Parser, ConsoleCommand)]
#[command(name = "spawn")]
struct SpawnCommand {
    count: usize,
    team: Team,
    object: Object,
}
impl SpawnCommand {
    pub fn update(
        mut log: ConsoleCommand<SpawnCommand>,
        mut commands: ObjectCommands,
        cursor: Query<&GlobalTransform, With<Cursor>>,
    ) {
        if let Some(Ok(SpawnCommand {
            object,
            team,
            count,
        })) = log.take()
        {
            reply!(log, "spawning {} {:?}", count, object);
            let cursor_position = cursor.single().translation().xy();
            let sqrt_count = (count as f32).sqrt() as usize;
            for i in 0..sqrt_count {
                for j in 0..sqrt_count {
                    commands.spawn(ObjectSpec {
                        object,
                        team,
                        position: cursor_position
                            + Vec2 {
                                x: (i * 10) as f32,
                                y: (j * 10) as f32,
                            },
                        ..default()
                    })
                }
            }
        }
    }
}
