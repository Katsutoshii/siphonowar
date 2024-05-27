use crate::prelude::*;
use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
    utils::HashMap,
};

pub struct AudioManagerPlugin;
impl Plugin for AudioManagerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpatialAudioManager>()
            .add_systems(Startup, SpatialAudioManager::setup)
            .add_systems(
                Update,
                SpatialAudioManager::update.after(AudioEvent::update),
            );
    }
}

#[derive(Component, Default)]
pub struct AudioEmitter {
    pub sample: AudioSample,
}

#[derive(Bundle, Default)]
pub struct AudioEmitterBundle {
    pub emitter: AudioEmitter,
    pub spatial: SpatialBundle,
    pub audio: AudioBundle,
}
impl AudioEmitterBundle {
    const DEFAULT_SETTINGS: PlaybackSettings = PlaybackSettings {
        spatial: true,
        paused: true,
        ..PlaybackSettings::ONCE
    };

    pub fn new(sample: AudioSample, assets: &AudioAssets) -> Self {
        match sample {
            AudioSample::Underwater => Self {
                emitter: AudioEmitter {
                    sample: AudioSample::Underwater,
                },
                audio: AudioBundle {
                    source: assets.samples[&AudioSample::Underwater].clone(),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        ..Self::DEFAULT_SETTINGS
                    },
                },
                ..default()
            },
            AudioSample::Punch => Self {
                emitter: AudioEmitter {
                    sample: AudioSample::Punch,
                },
                audio: AudioBundle {
                    source: assets.samples[&AudioSample::Punch].clone(),
                    settings: PlaybackSettings {
                        volume: Volume::new(0.4),
                        ..Self::DEFAULT_SETTINGS
                    },
                },
                ..default()
            },
            AudioSample::Pop(i) => Self {
                emitter: AudioEmitter {
                    sample: AudioSample::RandomPop,
                },
                audio: AudioBundle {
                    source: assets.samples[&AudioSample::Pop(i)].clone(),
                    settings: PlaybackSettings {
                        ..Self::DEFAULT_SETTINGS
                    },
                },
                ..default()
            },

            _ => Self { ..default() },
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct SpatialAudioManager {
    pub samplers: HashMap<AudioSample, AudioSampler>,
}
impl SpatialAudioManager {
    pub fn setup(mut commands: Commands, assets: Res<AudioAssets>) {
        let underwater = commands
            .spawn(AudioEmitterBundle::new(AudioSample::Underwater, &assets))
            .id();
        let punches: Vec<Entity> = (0..3)
            .map(|_| {
                commands
                    .spawn(AudioEmitterBundle::new(AudioSample::Punch, &assets))
                    .id()
            })
            .collect();
        let pops: Vec<Entity> = (0..14)
            .map(|i| {
                commands
                    .spawn(AudioEmitterBundle::new(
                        AudioSample::Pop(i % 7 + 1),
                        &assets,
                    ))
                    .id()
            })
            .collect();
        commands
            .spawn(SpatialAudioManagerBundle::default())
            .push_children(&[underwater])
            .push_children(&pops)
            .insert(SpatialAudioManager {
                samplers: [
                    (AudioSample::Underwater, AudioSampler::Single(underwater)),
                    (
                        AudioSample::RandomPop,
                        AudioSampler::Random(RandomSampler {
                            available: pops.into_iter().collect(),
                        }),
                    ),
                    (
                        AudioSample::Punch,
                        AudioSampler::Random(RandomSampler {
                            available: punches.into_iter().collect(),
                        }),
                    ),
                ]
                .into_iter()
                .collect(),
            });
    }

    pub fn get_sample(&mut self, sample: AudioSample) -> Option<Entity> {
        if let Some(sampler) = self.samplers.get_mut(&sample) {
            sampler.get_sample()
        } else {
            None
        }
    }

    pub fn free(&mut self, entity: Entity, sample: AudioSample) {
        if let Some(sampler) = self.samplers.get_mut(&sample) {
            sampler.free(entity);
        }
    }

    /// Clean up finished samples.
    pub fn update(
        mut manager: Query<&mut SpatialAudioManager>,
        sinks: Query<(Entity, &SpatialAudioSink, &AudioEmitter)>,
        mut commands: Commands,
    ) {
        let mut manager = manager.single_mut();
        for (entity, sink, emitter) in sinks.iter() {
            if sink.empty() {
                manager.free(entity, emitter.sample);
                commands.entity(entity).remove::<SpatialAudioSink>();
            }
        }
    }
}

#[derive(Bundle)]
pub struct SpatialAudioManagerBundle {
    pub name: Name,
    pub manager: SpatialAudioManager,
    pub listener: SpatialListener,
    pub spatial: SpatialBundle,
}
impl Default for SpatialAudioManagerBundle {
    fn default() -> Self {
        Self {
            name: Name::new("SpatialAudioManager"),
            manager: SpatialAudioManager::default(),
            listener: SpatialListener::new(2.),
            spatial: SpatialBundle::default(),
        }
    }
}
