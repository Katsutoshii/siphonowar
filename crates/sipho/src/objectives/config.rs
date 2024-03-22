use crate::prelude::*;

#[derive(Debug, Clone, Reflect)]
pub struct ObjectiveConfig {
    pub repell_radius: f32,
    pub slow_factor: f32,
    pub attack_radius: f32,
}
impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            repell_radius: 1.0,
            slow_factor: 0.0,
            attack_radius: 32.0,
        }
    }
}
impl ObjectiveConfig {
    /// Apply a slowing force against current velocity when near the goal.
    /// Also, undo some of the acceleration force when near the goal.
    pub fn slow_force(
        &self,
        velocity: Velocity,
        position: Vec2,
        target_position: Vec2,
        flow_acceleration: Acceleration,
    ) -> Acceleration {
        let position_delta = target_position - position;
        let dist_squared = position_delta.length_squared();
        let radius = self.repell_radius;
        let radius_squared = radius * radius;

        //  When within radius, this is negative
        let radius_diff = (dist_squared - radius_squared) / radius_squared;
        Acceleration(
            self.slow_factor
                * if dist_squared < radius_squared {
                    -1.0 * velocity.0
                } else {
                    Vec2::ZERO
                }
                + flow_acceleration.0 * radius_diff.clamp(-1., 0.),
        )
    }
}
