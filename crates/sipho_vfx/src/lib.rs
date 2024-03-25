pub mod fireworks;
use fireworks::FireworkPlugin;
use sipho_core::prelude::*;

pub mod prelude {
    pub use crate::{
        fireworks::{EffectCommands, FireworkSpec},
        VfxPlugin, VfxSize,
    };
    pub use sipho_core::prelude::*;
}

/// Represents size of an effect.
pub enum VfxSize {
    Tiny,
    Small,
    Medium,
}

/// Plugin for effects.
pub struct VfxPlugin;
impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FireworkPlugin);
    }
}
