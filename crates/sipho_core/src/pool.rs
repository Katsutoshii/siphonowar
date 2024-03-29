use crate::prelude::*;

#[derive(Resource, Deref, DerefMut)]
pub struct EntityPool<const N: usize> {
    #[deref]
    entities: [Entity; N],
    next: usize,
}
impl<const N: usize> Default for EntityPool<N> {
    fn default() -> Self {
        Self {
            entities: [Entity::PLACEHOLDER; N],
            next: 0,
        }
    }
}
impl<const N: usize> EntityPool<N> {
    pub fn new(entities: [Entity; N]) -> Self {
        Self { entities, next: 0 }
    }
    pub fn take(&mut self) -> Entity {
        let entity = self.entities[self.next];
        self.next = (self.next + 1) % N;
        entity
    }
}
