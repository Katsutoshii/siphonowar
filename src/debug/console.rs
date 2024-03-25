use bevy_console::{reply, AddConsoleCommand, ConsoleCommand, ConsolePlugin};
use clap::Parser;
use sipho::{prelude::*, scene::SaveEvent};

/// Plugin for input action events.
pub struct CustomConsolePlugin;
impl Plugin for CustomConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin)
            .add_console_command::<SpawnCommand, _>(SpawnCommand::update.in_set(SystemStage::Spawn))
            .add_console_command::<DespawnCommand, _>(
                DespawnCommand::update.in_set(SystemStage::Despawn),
            )
            .add_console_command::<SaveCommand, _>(
                SaveCommand::update.in_set(SystemStage::Despawn),
            );
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
                                x: (i * 40) as f32,
                                y: (j * 40) as f32,
                            },
                        ..default()
                    })
                }
            }
        }
    }
}

/// Example command
#[derive(Parser, ConsoleCommand)]
#[command(name = "despawn")]
struct DespawnCommand {
    team: Team,
}
impl DespawnCommand {
    pub fn update(
        mut log: ConsoleCommand<DespawnCommand>,
        mut commands: ObjectCommands,
        objects: Query<(Entity, &GridEntity, &Team)>,
    ) {
        if let Some(Ok(DespawnCommand { team })) = log.take() {
            reply!(log, "despawn {:?}", team);
            for (entity, grid_entity, object_team) in objects.iter() {
                if *object_team == team {
                    commands.despawn(entity, *grid_entity);
                }
            }
        }
    }
}
#[derive(Parser, ConsoleCommand)]
#[command(name = "save")]
struct SaveCommand {
    pub path: String,
}
impl SaveCommand {
    pub fn update(mut log: ConsoleCommand<SaveCommand>, mut events: EventWriter<SaveEvent>) {
        if let Some(Ok(SaveCommand { path })) = log.take() {
            reply!(log, "saving to {}", path);
            events.send(SaveEvent { path });
        }
    }
}
