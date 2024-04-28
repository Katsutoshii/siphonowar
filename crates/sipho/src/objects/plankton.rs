use rand::Rng;

use crate::prelude::*;

pub struct PlanktonPlugin;
impl Plugin for PlanktonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (Plankton::spawn
                .in_set(FixedUpdateStage::Spawn)
                .in_set(GameStateSet::Running),),
        )
        .add_systems(OnExit(GameState::Loading), Plankton::spawn_random);
    }
}

#[derive(Component, Default)]
pub struct Plankton;
impl Plankton {
    pub fn spawn_random(grid_spec: Res<GridSpec>, mut commands: ObjectCommands) {
        let bounds = grid_spec.world2d_bounds_eps();
        for _ in 0..1000 {
            commands.spawn(ObjectSpec {
                object: Object::Plankton,
                team: Team::None,
                position: Vec2 {
                    x: rand::thread_rng().gen_range(bounds.min.x..bounds.max.x),
                    y: rand::thread_rng().gen_range(bounds.min.y..bounds.max.y),
                },
                ..default()
            });
        }
    }
    pub fn spawn(mut control_events: EventReader<ControlEvent>, mut commands: ObjectCommands) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::SpawnPlankton) {
                commands.spawn(ObjectSpec {
                    object: Object::Plankton,
                    team: Team::None,
                    position: control_event.position,
                    ..default()
                });
            }
        }
    }
}
