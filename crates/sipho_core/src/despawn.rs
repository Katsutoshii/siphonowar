use crate::prelude::*;

pub struct DespawnPlugin;
impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnEvent>().add_systems(
            FixedUpdate,
            DespawnEvent::update
                .run_if(on_event::<DespawnEvent>())
                .in_set(SystemStage::Despawn),
        );
    }
}

#[derive(Debug, Event)]
pub struct DespawnEvent(pub Entity);
impl DespawnEvent {
    pub fn update(mut events: EventReader<DespawnEvent>, mut commands: Commands) {
        for event in events.read() {
            // dbg!(event);
            commands.entity(event.0).despawn_recursive();
        }
    }
}
