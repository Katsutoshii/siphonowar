use crate::prelude::*;
use crate::zooid_head::NearestZooidHead;

pub struct ZooidWorkerPlugin;
impl Plugin for ZooidWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (ZooidWorker::debug_spawn.in_set(FixedUpdateStage::Spawn),)
                .in_set(GameStateSet::Running),
        );
    }
}

#[derive(Bundle, Default)]
pub struct WorkerBundle {
    pub worker: ZooidWorker,
    pub nearest_head: NearestZooidHead,
    pub object: ObjectBundle,
}

/// State for an individual zooid.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct ZooidWorker {
    pub theta: f32,
}
impl Default for ZooidWorker {
    fn default() -> Self {
        Self { theta: 0.0 }
    }
}
impl ZooidWorker {
    pub fn debug_spawn(
        mut commands: ObjectCommands,
        mut control_events: EventReader<ControlEvent>,
    ) {
        for control_event in control_events.read() {
            let team: Option<Team> = if control_event.is_pressed(ControlAction::SpawnBlue) {
                Some(Team::Blue)
            } else if control_event.is_pressed(ControlAction::SpawnRed) {
                Some(Team::Red)
            } else {
                None
            };
            if let Some(team) = team {
                commands.spawn(ObjectSpec {
                    object: Object::Worker,
                    team,
                    position: control_event.position,
                    ..default()
                });
            }
        }
    }
}
