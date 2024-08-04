use rand::Rng;

use crate::{prelude::*, terrain::Terrain};

pub struct PlanktonPlugin;
impl Plugin for PlanktonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (Plankton::spawn
                .in_set(FixedUpdateStage::Spawn)
                .in_set(GameStateSet::Running),),
        )
        .add_systems(
            OnExit(GameState::Loading),
            Plankton::spawn_random.after(Terrain::setup_obstacles),
        );
    }
}

#[derive(Bundle, Default)]
pub struct PlanktonBundle {
    pub plankton: Plankton,
    pub object: ObjectBundle,
}

#[derive(Component, Default)]
pub struct Plankton;
impl Plankton {
    pub fn spawn_random(
        grid_spec: Res<GridSpec>,
        mut commands: ObjectCommands,
        obstacles: Res<Grid2<Obstacle>>,
    ) {
        let bounds = grid_spec.world2d_bounds_eps();
        for _ in 0..5000 {
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
    }
    pub fn spawn(mut control_events: EventReader<ControlEvent>, mut commands: ObjectCommands) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::Plankton) {
                commands.spawn(ObjectSpec {
                    object: Object::Plankton,
                    team: Team::None,
                    position: Position(control_event.position),
                    ..default()
                });
            }
        }
    }
}
