use bevy::prelude::*;

use rand::{thread_rng, Rng};

#[derive(Reflect)]
pub enum AudioSampler {
    Single(Entity),
    Random(RandomSampler),
}
impl AudioSampler {
    pub fn get_sample(&mut self) -> Option<Entity> {
        match self {
            Self::Single(entity) => Some(*entity),
            Self::Random(sampler) => sampler.get_sample(),
        }
    }

    pub fn free(&mut self, entity: Entity) {
        if let Self::Random(sampler) = self {
            sampler.free(entity)
        }
    }
}

#[derive(Reflect, Default)]
pub struct RandomSampler {
    pub available: Vec<Entity>,
}
impl RandomSampler {
    pub fn get_sample(&mut self) -> Option<Entity> {
        self.available.remove_random(&mut thread_rng())
    }

    pub fn free(&mut self, entity: Entity) {
        self.available.push(entity);
    }
}

pub trait RemoveRandom {
    type Item;

    fn remove_random<R: Rng>(&mut self, rng: &mut R) -> Option<Self::Item>;
}

impl<T> RemoveRandom for Vec<T> {
    type Item = T;

    fn remove_random<R: Rng>(&mut self, rng: &mut R) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let index = rng.gen_range(0..self.len());
            Some(self.swap_remove(index))
        }
    }
}
