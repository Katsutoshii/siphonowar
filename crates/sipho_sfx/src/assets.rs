use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};

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
    pub fn get_path(self) -> &'static str {
        match self {
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
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct AudioAssets {
    pub samples: HashMap<AudioSample, Handle<AudioSource>>,
}
impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let result = Self {
            samples: [
                AudioSample::Underwater,
                AudioSample::Punch,
                AudioSample::Snap,
                AudioSample::Pop(1),
                AudioSample::Pop(2),
                AudioSample::Pop(3),
                AudioSample::Pop(4),
                AudioSample::Pop(5),
                AudioSample::Pop(6),
                AudioSample::Pop(7),
                AudioSample::Bubble(1),
                AudioSample::Bubble(2),
                AudioSample::Bubble(3),
                AudioSample::Zap(1),
                AudioSample::Zap(2),
                AudioSample::Zap(3),
            ]
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
