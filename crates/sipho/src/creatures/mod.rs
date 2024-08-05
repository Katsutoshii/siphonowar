mod snake;

use rand::Rng;

use crate::{prelude::*, terrain::Terrain};

pub struct CreaturePlugin;
impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCreatureEvent>()
            .add_systems(
                FixedUpdate,
                SpawnCreatureEvent::update
                    .in_set(GameStateSet::Running)
                    .in_set(FixedUpdateStage::Spawn),
            )
            .add_systems(
                OnExit(GameState::Loading),
                spawn_random.after(Terrain::setup_obstacles),
            );
    }
}

pub fn spawn_random(
    grid_spec: Res<GridSpec>,
    mut commands: ObjectCommands,
    obstacles: Res<Grid2<Obstacle>>,
    mut elastics: EventWriter<SpawnElasticEvent>,
) {
    let bounds = grid_spec.world2d_bounds_eps();

    // Spawn plankton
    for _ in 0..3500 {
        let position = Position::new(
            rand::thread_rng().gen_range(bounds.min.x..bounds.max.x),
            rand::thread_rng().gen_range(bounds.min.y..bounds.max.y),
        );
        let rowcol = obstacles.to_rowcol(position.0).unwrap();
        if !obstacles.is_clear(rowcol) {
            continue;
        }

        commands.spawn(ObjectSpec {
            object: Object::Plankton,
            team: Team::None,
            position,
            ..default()
        });
    }

    // Spawn snakes
    for _ in 0..150 {
        let position = Position::new(
            rand::thread_rng().gen_range(bounds.min.x..bounds.max.x),
            rand::thread_rng().gen_range(bounds.min.y..bounds.max.y),
        );
        snake::spawn_snake(position, &mut commands, &mut elastics);
    }
    // Spawn gemstones
    for _ in 0..150 {
        let position = Position::new(
            rand::thread_rng().gen_range(bounds.min.x..bounds.max.x),
            rand::thread_rng().gen_range(bounds.min.y..bounds.max.y),
        );
        commands.spawn(ObjectSpec {
            object: Object::GemStone,
            team: Team::None,
            position,
            ..default()
        });
    }
}

pub enum CreatureType {
    Snake,
}

#[derive(Event)]
pub struct SpawnCreatureEvent {
    pub creature_type: CreatureType,
    pub position: Position,
}
impl SpawnCreatureEvent {
    pub fn update(
        mut events: EventReader<SpawnCreatureEvent>,
        mut commands: ObjectCommands,
        mut elastics: EventWriter<SpawnElasticEvent>,
    ) {
        for event in events.read() {
            snake::spawn_snake(event.position, &mut commands, &mut elastics);
        }
    }
}
