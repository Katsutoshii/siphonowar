use crate::prelude::*;
use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
    utils::HashMap,
};

pub struct AudioAssetsPlugin;
impl Plugin for AudioAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AudioSample>()
            .register_type::<HashMap<AudioSample, Handle<AudioSource>>>()
            .register_type::<AudioAssets>()
            .init_resource::<AudioAssets>();
    }
}

#[derive(PartialEq, Eq, Hash, Reflect, Default, Clone, Copy, Debug)]
pub enum AudioSample {
    #[default]
    None,
    TranceIntro,
    Trance,
    Underwater,
    Snap,
    Punch,
    RandomPop,
    Pop(u8),
    RandomBubble,
    Bubble(u8),
    RandomZap,
    Zap(u8),
}
impl AudioSample {
    pub const ALL: [Self; 18] = [
        Self::TranceIntro,
        Self::Trance,
        Self::Underwater,
        Self::Punch,
        Self::Snap,
        Self::Pop(1),
        Self::Pop(2),
        Self::Pop(3),
        Self::Pop(4),
        Self::Pop(5),
        Self::Pop(6),
        Self::Pop(7),
        Self::Bubble(1),
        Self::Bubble(2),
        Self::Bubble(3),
        Self::Zap(1),
        Self::Zap(2),
        Self::Zap(3),
    ];
    pub const SINGLES: [Self; 3] = [Self::TranceIntro, Self::Trance, Self::Underwater];
    pub fn get_path(self) -> &'static str {
        match self {
            Self::TranceIntro => "sounds/bgm/trance-intro.ogg",
            Self::Trance => "sounds/bgm/trance-loop.ogg",
            Self::Underwater => "sounds/ambience/underwater.ogg",
            Self::Punch => "sounds/punch.ogg",
            Self::Snap => "sounds/snap.ogg",
            Self::Pop(1) => "sounds/pops/pop(1).ogg",
            Self::Pop(2) => "sounds/pops/pop(2).ogg",
            Self::Pop(3) => "sounds/pops/pop(3).ogg",
            Self::Pop(4) => "sounds/pops/pop(4).ogg",
            Self::Pop(5) => "sounds/pops/pop(5).ogg",
            Self::Pop(6) => "sounds/pops/pop(6).ogg",
            Self::Pop(7) => "sounds/pops/pop(7).ogg",
            Self::Pop(_) | Self::RandomPop => unreachable!(),
            Self::Bubble(1) => "sounds/bubbles/bubble(1).ogg",
            Self::Bubble(2) => "sounds/bubbles/bubble(2).ogg",
            Self::Bubble(3) => "sounds/bubbles/bubble(3).ogg",
            Self::Bubble(_) | Self::RandomBubble => unreachable!(),
            Self::Zap(1) => "sounds/zaps/zap(1).ogg",
            Self::Zap(2) => "sounds/zaps/zap(2).ogg",
            Self::Zap(3) => "sounds/zaps/zap(3).ogg",
            Self::Zap(_) | Self::RandomZap => unreachable!(),
            Self::None => unreachable!(),
        }
    }

    const DEFAULT_SETTINGS: PlaybackSettings = PlaybackSettings {
        spatial: true,
        paused: true,
        ..PlaybackSettings::ONCE
    };

    pub fn get_settings(self) -> PlaybackSettings {
        match self {
            Self::TranceIntro => PlaybackSettings {
                volume: Volume::new(0.13),
                mode: PlaybackMode::Once,
                ..Self::DEFAULT_SETTINGS
            },
            Self::Trance => PlaybackSettings {
                volume: Volume::new(0.13),
                mode: PlaybackMode::Loop,
                ..Self::DEFAULT_SETTINGS
            },
            Self::Underwater => PlaybackSettings {
                volume: Volume::new(1.2),
                mode: PlaybackMode::Loop,
                ..Self::DEFAULT_SETTINGS
            },
            AudioSample::Punch => PlaybackSettings {
                volume: Volume::new(0.85),
                ..Self::DEFAULT_SETTINGS
            },
            AudioSample::Pop(_) => PlaybackSettings {
                volume: Volume::new(0.6),
                ..Self::DEFAULT_SETTINGS
            },
            AudioSample::Bubble(_) => PlaybackSettings {
                volume: Volume::new(1.0),
                ..Self::DEFAULT_SETTINGS
            },
            AudioSample::Zap(_) => PlaybackSettings {
                volume: Volume::new(0.35),
                ..Self::DEFAULT_SETTINGS
            },
            _ => default(),
        }
    }

    pub fn get_canonical(self) -> Self {
        match self {
            Self::Pop(_) => Self::RandomPop,
            Self::Bubble(_) => Self::RandomBubble,
            Self::Zap(_) => Self::RandomZap,
            x => x,
        }
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct AudioAssets {
    pub samples: HashMap<AudioSample, Handle<AudioSource>>,
}
impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let result = Self {
            samples: AudioSample::ALL
                .map(|s| (s, world.load_asset(s.get_path())))
                .into_iter()
                .collect(),
        };
        let mut load_state = world.resource_mut::<AssetLoadState>();
        for (_sample, handle) in &result.samples {
            load_state.track(handle);
        }
        result
    }
}
