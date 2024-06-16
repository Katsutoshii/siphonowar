use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};

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
    pub next: Option<AudioEvent>,
}

#[derive(Bundle, Default)]
pub struct AudioEmitterBundle {
    pub emitter: AudioEmitter,
    pub spatial: SpatialBundle,
    pub audio: AudioBundle,
}
impl AudioEmitterBundle {
    pub fn new(sample: AudioSample, assets: &AudioAssets) -> Self {
        Self {
            audio: AudioBundle {
                source: assets.samples[&sample].clone(),
                settings: sample.get_settings(),
            },
            emitter: AudioEmitter {
                sample: sample.get_canonical(),
                ..default()
            },
            ..default()
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
        let emitters = {
            let mut emitters = HashMap::new();
            for sample in AudioSample::SINGLES {
                emitters.insert(
                    sample,
                    commands
                        .spawn(AudioEmitterBundle::new(sample, &assets))
                        .id(),
                );
            }
            emitters
        };
        let multi_emitters = {
            let mut emitters: HashMap<AudioSample, Vec<Entity>> = HashMap::new();
            emitters.insert(
                AudioSample::RandomPop,
                (0..14)
                    .map(|i| {
                        commands
                            .spawn(AudioEmitterBundle::new(
                                AudioSample::Pop(i % 7 + 1),
                                &assets,
                            ))
                            .id()
                    })
                    .collect(),
            );
            emitters.insert(
                AudioSample::RandomBubble,
                (0..6)
                    .map(|i| {
                        commands
                            .spawn(AudioEmitterBundle::new(
                                AudioSample::Bubble(i % 3 + 1),
                                &assets,
                            ))
                            .id()
                    })
                    .collect(),
            );
            emitters.insert(
                AudioSample::Punch,
                (0..3)
                    .map(|_| {
                        commands
                            .spawn(AudioEmitterBundle::new(AudioSample::Punch, &assets))
                            .id()
                    })
                    .collect(),
            );
            emitters.insert(
                AudioSample::RandomZap,
                (0..6)
                    .map(|i| {
                        commands
                            .spawn(AudioEmitterBundle::new(
                                AudioSample::Zap(i % 3 + 1),
                                &assets,
                            ))
                            .id()
                    })
                    .collect(),
            );
            emitters
        };

        let mut samplers = HashMap::new();
        let mut parent = commands.spawn(SpatialAudioManagerBundle::default());

        for (sample, emitter) in emitters {
            samplers.insert(sample, AudioSampler::Single(emitter));
            parent.push_children(&[emitter]);
        }
        for (sample, emitters) in multi_emitters {
            parent.push_children(&emitters);
            samplers.insert(
                sample,
                AudioSampler::Random(RandomSampler {
                    available: emitters.into_iter().collect(),
                }),
            );
        }

        parent.insert(SpatialAudioManager { samplers });
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
        mut sinks: Query<(Entity, &SpatialAudioSink, &mut AudioEmitter)>,
        mut commands: Commands,
        mut audio: EventWriter<AudioEvent>,
    ) {
        let mut manager = manager.single_mut();
        for (entity, sink, mut emitter) in sinks.iter_mut() {
            if sink.empty() {
                manager.free(entity, emitter.sample);
                commands.entity(entity).remove::<SpatialAudioSink>();
                if let Some(event) = emitter.next.take() {
                    audio.send(event.clone());
                }
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
