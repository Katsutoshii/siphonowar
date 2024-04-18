use crate::prelude::*;

pub struct PlanktonPlugin;
impl Plugin for PlanktonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (Plankton::spawn
                .in_set(SystemStage::ObjectSpawn)
                .in_set(GameStateSet::Running),),
        );
    }
}

#[derive(Component, Default)]
pub struct Plankton;
impl Plankton {
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
