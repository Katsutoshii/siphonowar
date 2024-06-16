use crate::prelude::*;

pub struct AmbiencePlugin;
impl Plugin for AmbiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), play_ambience);
    }
}

fn play_ambience(mut audio: EventWriter<AudioEvent>) {
    audio.send(AudioEvent {
        sample: AudioSample::Underwater,
        ..default()
    });
    audio.send(AudioEvent {
        sample: AudioSample::TranceIntro,
        queue: vec![AudioSample::Trance],
        ..default()
    });
}
