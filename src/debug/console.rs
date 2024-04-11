use bevy_console::{reply, AddConsoleCommand, ConsoleCommand, ConsolePlugin};
use clap::Parser;
use sipho::{prelude::*, scene::SaveEvent};

/// Plugin for input action events.
pub struct CustomConsolePlugin;
impl Plugin for CustomConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin)
            .add_console_command::<SpawnCommand, _>(SpawnCommand::update.in_set(SystemStage::Spawn))
            .add_console_command::<BattleCommand, _>(
                BattleCommand::update.in_set(SystemStage::Spawn),
            )
            .add_console_command::<DespawnCommand, _>(
                DespawnCommand::update.in_set(SystemStage::Death),
            )
            .add_console_command::<SaveCommand, _>(SaveCommand::update.in_set(SystemStage::Spawn));
    }
}

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
        cursor: CursorParam,
        raycast: RaycastCommands,
        obstacles: ResMut<Grid2<Obstacle>>,
    ) {
        if let Some(Ok(SpawnCommand {
            object,
            team,
            count,
        })) = log.take()
        {
            reply!(log, "spawning {} {:?}", count, object);
            if let Some(ray) = cursor.ray3d() {
                if let Some(raycast_event) = raycast.raycast(ray) {
                    let sqrt_count = (count as f32).sqrt() as usize;
                    for i in 0..sqrt_count {
                        for j in 0..sqrt_count {
                            let position = raycast_event.world_position
                                + Vec2 {
                                    x: (i * 40) as f32,
                                    y: (j * 40) as f32,
                                };
                            if obstacles[obstacles.to_rowcol(position)] == Obstacle::Empty {
                                commands.spawn(ObjectSpec {
                                    object,
                                    team,
                                    position,
                                    ..default()
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "battle")]
struct BattleCommand {
    count: usize,
}
impl BattleCommand {
    pub fn update(
        mut log: ConsoleCommand<BattleCommand>,
        mut commands: ObjectCommands,
        cursor: CursorParam,
        raycast: RaycastCommands,
        obstacles: ResMut<Grid2<Obstacle>>,
    ) {
        if let Some(Ok(BattleCommand { count })) = log.take() {
            reply!(log, "spawning battle {}", count);
            if let Some(ray) = cursor.ray3d() {
                if let Some(raycast_event) = raycast.raycast(ray) {
                    let sqrt_count = (count as f32).sqrt() as usize;
                    for i in 0..sqrt_count {
                        for j in 0..sqrt_count {
                            let stride = 40;
                            let blue_position = raycast_event.world_position
                                + Vec2 {
                                    x: (i * stride) as f32,
                                    y: (j * stride) as f32,
                                };
                            if obstacles[obstacles.to_rowcol(blue_position)] == Obstacle::Empty {
                                commands.spawn(ObjectSpec {
                                    object: Object::Worker,
                                    team: Team::Blue,
                                    position: blue_position,
                                    ..default()
                                })
                            }
                            let red_position = raycast_event.world_position
                                + Vec2 {
                                    x: (i * stride + stride / 2) as f32,
                                    y: (j * stride + stride / 2) as f32,
                                };
                            if obstacles[obstacles.to_rowcol(red_position)] == Obstacle::Empty {
                                commands.spawn(ObjectSpec {
                                    object: Object::Worker,
                                    team: Team::Red,
                                    position: red_position,
                                    ..default()
                                })
                            }
                        }
                    }
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
        objects: Query<(Entity, &Team)>,
    ) {
        if let Some(Ok(DespawnCommand { team })) = log.take() {
            reply!(log, "despawn {:?}", team);
            for (entity, object_team) in objects.iter() {
                if *object_team == team {
                    commands.despawn(entity);
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
