use crate::prelude::*;

#[derive(Debug, Clone, Reflect)]
pub struct ObjectiveConfig {
    pub repell_radius: f32,
    pub slow_factor: f32,
}
impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            repell_radius: 1.0,
            slow_factor: 0.0,
        }
    }
}
