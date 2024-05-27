use bevy::prelude::*;
pub mod assets;
pub mod events;
pub mod manager;
pub mod sampler;

pub struct SiphoSfxPlugin;
impl Plugin for SiphoSfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            assets::AudioAssetsPlugin,
            events::AudioEventsPlugin,
            manager::AudioManagerPlugin,
        ));
    }
}

pub mod prelude {
    pub use crate::{
        assets::{AudioAssets, AudioSample},
        events::AudioEvent,
        manager::AudioEmitter,
        manager::SpatialAudioManager,
        sampler::{AudioSampler, RandomSampler},
        SiphoSfxPlugin,
    };
    pub use sipho_core::prelude::*;
}
