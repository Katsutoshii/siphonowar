use crate::prelude::*;
use bevy::prelude::*;

pub struct AudioEventsPlugin;
impl Plugin for AudioEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AudioEvent>()
            .add_systems(Update, AudioEvent::update);
    }
}

#[derive(Event, Default)]
pub struct AudioEvent {
    pub position: Option<Vec2>,
    pub sample: AudioSample,
}
impl AudioEvent {
    pub fn update(
        mut events: EventReader<AudioEvent>,
        camera: Query<&GlobalTransform, With<MainCamera>>,
        mut manager: Query<&mut SpatialAudioManager>,
        mut emitters: Query<(&AudioEmitter, &mut SpatialAudioSink, &mut Transform)>,
        // mut commands: Commands,
    ) {
        let camera_position = camera.single().translation();
        let y_offset = Vec2::Y * MainCamera::y_offset(camera_position.z);

        let mut manager = manager.single_mut();
        for event in events.read() {
            let scaled_position = if let Some(position) = event.position {
                (position - camera_position.xy() - y_offset) / 850.
            } else {
                Vec2::ZERO
            };
            if scaled_position.x.abs() > 2.5 || scaled_position.y.abs() > 2.5 {
                continue;
            }

            if let Some(entity) = manager.get_sample(event.sample) {
                if let Ok((_emitter, sink, mut transform)) = emitters.get_mut(entity) {
                    transform.translation.x = scaled_position.x;
                    transform.translation.y = scaled_position.y;
                    sink.play();
                } else {
                    error!("Missing audio sink.");
                    manager.free(entity, event.sample);
                }
            }
        }
    }
}
