use crate::prelude::*;

pub struct DespawnPlugin;
impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnEvent>().add_systems(
            FixedUpdate,
            (DespawnEvent::update
                .run_if(on_event::<DespawnEvent>())
                .in_set(FixedUpdateStage::Despawn),),
        );
    }
}

#[derive(Debug, Event)]
pub struct DespawnEvent(pub Entity);
impl DespawnEvent {
    pub fn update(mut events: EventReader<DespawnEvent>, mut commands: Commands) {
        for event in events.read() {
            commands.entity(event.0).despawn_recursive();
        }
    }
}

/// Schedule despawn for a particle.
#[derive(Component, Deref, DerefMut, Default)]
pub struct ScheduleDespawn(pub Timer);
impl ScheduleDespawn {
    pub fn despawn(
        mut query: Query<(Entity, &mut ScheduleDespawn)>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        for (entity, mut timer) in &mut query {
            timer.tick(time.delta());
            if timer.finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
