use sipho_core::prelude::*;

pub mod bubbles;
pub mod fireworks;
pub mod lightning;

pub mod prelude {
    pub use crate::{
        fireworks::{FireworkColor, FireworkCommands, FireworkSpec},
        lightning::{Lightning, LightningCommands},
        VfxPlugin, VfxSize,
    };
    pub use sipho_core::prelude::*;
}

/// Represents size of an effect.
#[derive(Debug, Clone, Copy)]
pub enum VfxSize {
    Small,
    Medium,
}

/// Plugin for effects.
pub struct VfxPlugin;
impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            lightning::LightningPlugin,
            fireworks::FireworkPlugin,
            bubbles::BubblesPlugin,
        ));
    }
}
