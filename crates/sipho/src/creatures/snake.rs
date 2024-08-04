use crate::prelude::*;
use bevy::prelude::*;

pub fn spawn_snake(
    position: Position,
    commands: &mut ObjectCommands,
    elastics: &mut EventWriter<SpawnElasticEvent>,
) -> Option<()> {
    let team = Team::None;
    let specs = vec![
        ObjectSpec {
            object: Object::Head,
            position: position + Position::new(0.0, 0.0),
            ..default()
        },
        ObjectSpec {
            object: Object::Plankton,
            position: position + Position::new(2.0, 10.0),
            ..default()
        },
        ObjectSpec {
            object: Object::Plankton,
            position: position + Position::new(-2.0, 20.0),
            ..default()
        },
        ObjectSpec {
            object: Object::Plankton,
            position: position + Position::new(0.0, 30.0),
            ..default()
        },
        ObjectSpec {
            object: Object::Plankton,
            position: position + Position::new(2.0, 40.0),
            ..default()
        },
        ObjectSpec {
            object: Object::Plankton,
            position: position + Position::new(-2.0, 50.0),
            ..default()
        },
    ];
    let entities = commands.spawn_batch(specs)?;
    for pair in entities.windows(2) {
        if let &[e1, e2] = pair {
            elastics.send(SpawnElasticEvent {
                elastic: Elastic((e1, e2)),
                team,
            });
        }
    }
    None
}
