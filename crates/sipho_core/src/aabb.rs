use std::mem::swap;

use bevy::prelude::*;

/// Axis-aligned bounding box in 2d.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct Aabb2 {
    pub min: Vec2,
    pub max: Vec2,
}
impl Aabb2 {
    pub fn enforce_minmax(&mut self) {
        if self.min.x > self.max.x {
            swap(&mut self.min.x, &mut self.max.x);
        }
        if self.min.y > self.max.y {
            swap(&mut self.min.y, &mut self.max.y);
        }
    }
    /// Returns the size of the bounding box.
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
    /// Returns the center of the bounding box.
    pub fn center(&self) -> Vec2 {
        (self.max + self.min) / 2.
    }
    // Returns true if point is in the bounding box.
    pub fn contains(&self, point: Vec2) -> bool {
        (self.min.x <= point.x && point.x < self.max.x)
            && (self.min.y <= point.y && point.y < self.max.y)
    }
    /// Clamp a 2d vector to the bounding box.
    pub fn clamp2(&self, vec: &mut Vec2) {
        vec.x = vec.x.clamp(self.min.x, self.max.x);
        vec.y = vec.y.clamp(self.min.y, self.max.y);
    }
    /// Clamp a 3d vector (ignoring Z) to the bounding box.
    pub fn clamp3(&self, vec: &mut Vec3) {
        vec.x = vec.x.clamp(self.min.x, self.max.x);
        vec.y = vec.y.clamp(self.min.y, self.max.y);
    }
}
